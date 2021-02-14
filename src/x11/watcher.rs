use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use snafu::ResultExt;
use x11rb::protocol::{xfixes, xproto, xproto::ConnectionExt, Event};

use crate::{
    error,
    pubsub::{self, Subscriber},
    x11::Context,
    ClipboardSubscribe, Error,
};

#[derive(Debug)]
pub struct Watcher {
    context: Arc<Context>,
    is_running: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<Result<(), Error>>>,
    subscriber: Subscriber,
}

impl Watcher {
    pub fn new(context: Context) -> Result<Watcher, Error> {
        let mode = context.mode;
        let (notifier, subscriber) = pubsub::new(mode);
        let is_running = Arc::new(AtomicBool::new(true));
        let context = Arc::new(context);
        let thread = thread::spawn({
            let context = context.clone();
            let is_running = is_running.clone();
            move || {
                while is_running.load(Ordering::Relaxed) {
                    match context.prepare_for_monitoring_event() {
                        Ok(_) => {}
                        Err(err) => {
                            notifier.close();
                            return Err(err);
                        }
                    }

                    let new_event = match context.wait_for_event() {
                        Ok(event) => event,
                        Err(err) => {
                            notifier.close();
                            return Err(err);
                        }
                    };

                    match new_event {
                        Event::XfixesSelectionNotify(_) => notifier.notify_all(),
                        Event::ClientMessage(event) => {
                            if context.is_close_event(&event) {
                                tracing::info!(
                                    "Close connection event received, X11 clipboard monitoring \
                                     thread is closing"
                                );
                                break;
                            }
                        }
                        _ => {}
                    }
                }

                notifier.close();
                Ok(())
            }
        });

        Ok(Watcher { context, is_running, thread: Some(thread), subscriber })
    }
}

impl ClipboardSubscribe for Watcher {
    type Subscriber = Subscriber;

    fn subscribe(&self) -> Result<Self::Subscriber, Error> { Ok(self.subscriber.clone()) }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Release);
        let _ = self.context.send_close_connection_event();
        self.thread.take().map(|t| t.join());
    }
}

impl Context {
    pub fn prepare_for_monitoring_event(&self) -> Result<(), Error> {
        const EXT_NAME: &str = "XFIXES";
        let xfixes = self
            .connection
            .query_extension(EXT_NAME.as_bytes())
            .with_context(|| error::QueryExtension { extension_name: EXT_NAME.to_string() })?
            .reply()
            .context(error::Reply)?;

        if !xfixes.present {
            return Err(Error::XfixesNotPresent);
        }

        {
            use xfixes::{ConnectionExt, SelectionEventMask};

            self.connection.xfixes_query_version(5, 0).context(error::QueryXfixesVersion)?;

            self.connection
                .xfixes_select_selection_input(
                    self.window,
                    self.clipboard_type(),
                    xproto::EventMask::NO_EVENT,
                )
                .context(error::SelectXfixesSelectionInput)?;

            self.connection
                .xfixes_select_selection_input(
                    self.window,
                    self.clipboard_type(),
                    SelectionEventMask::SET_SELECTION_OWNER
                        | SelectionEventMask::SELECTION_WINDOW_DESTROY
                        | SelectionEventMask::SELECTION_CLIENT_CLOSE,
                )
                .context(error::SelectXfixesSelectionInput)?;
        }

        self.flush_connection()?;
        Ok(())
    }
}
