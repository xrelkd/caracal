use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread,
};

use snafu::ResultExt;
use x11rb::{
    connection::Connection,
    protocol::{xproto, xproto::ConnectionExt, Event},
    wrapper::ConnectionExt as WrapperConnectionExt,
};

use crate::{
    error,
    x11::{self, Context},
    ClipboardStore, Error, MimeData,
};

type ClipBuffer = Arc<RwLock<Option<MimeData>>>;

pub const INCR_CHUNK_SIZE: usize = 4 * 1024;

struct IncrState {
    requestor: xproto::Atom,
    property: xproto::Atom,
    pos: usize,
}

impl IncrState {
    #[inline]
    fn new(requestor: xproto::Atom, property: xproto::Atom) -> IncrState {
        IncrState { requestor, property, pos: 0 }
    }
}

#[derive(Debug)]
pub struct Setter {
    context: Arc<Context>,
    clip_buffer: ClipBuffer,
    provider_thread: ProviderThread,
    is_running: Arc<AtomicBool>,
}

#[derive(Debug)]
struct ProviderThread(Option<thread::JoinHandle<Result<(), Error>>>);

impl Drop for Setter {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Release);
        let _ = self.context.send_close_connection_event();
        let _ = self.context.release_selection_ownership();
    }
}

impl Setter {
    pub fn new(context: Context) -> Result<Setter, Error> {
        let context = Arc::new(context);
        let is_running = Arc::new(AtomicBool::new(true));
        let clip_buffer = Arc::new(RwLock::new(None));
        let join_handle =
            ProviderThread::spawn(context.clone(), is_running.clone(), clip_buffer.clone());

        Ok(Setter { context, provider_thread: join_handle, is_running, clip_buffer })
    }
}

impl ClipboardStore for Setter {
    fn store(&self, mime: mime::Mime, data: &[u8]) -> Result<(), Error> {
        let data = MimeData::new(mime, data.to_vec());
        self.store_mime_data(data)?;
        Ok(())
    }

    fn store_mime_data(&self, mime_data: MimeData) -> Result<(), Error> {
        self.context.claim_selection_ownership()?;
        match self.clip_buffer.write() {
            Ok(mut write_clip) => {
                *write_clip = Some(mime_data);
                Ok(())
            }
            Err(_) => Err(Error::X11WorkerThreadFailed),
        }
    }

    fn clear(&self) -> Result<(), Error> {
        self.context.release_selection_ownership()?;
        Ok(())
    }
}

impl Drop for ProviderThread {
    fn drop(&mut self) { self.0.take().map(|j| j.join()); }
}

impl ProviderThread {
    fn spawn(
        context: Arc<Context>,
        is_running: Arc<AtomicBool>,
        clip_buffer: ClipBuffer,
    ) -> ProviderThread {
        let handle = thread::spawn(move || {
            let mut transfer_states: HashMap<xproto::Atom, IncrState> = HashMap::new();
            let targets_atom = x11::get_intern_atom(&context.connection, "TARGETS")?;
            let max_length = context.connection.setup().maximum_request_length as usize * 2;

            while is_running.load(Ordering::Relaxed) {
                let new_event = context.connection.wait_for_event().context(error::WaitForEvent)?;

                match new_event {
                    Event::ClientMessage(event) => {
                        if context.is_close_event(&event) {
                            tracing::info!(
                                "Close connection event received, X11 worker thread is closing"
                            );
                            return Ok(());
                        }
                    }
                    Event::SelectionClear(_event) => {
                        tracing::debug!("Lost selection ownership");
                        transfer_states.clear();
                    }
                    Event::SelectionRequest(event) => {
                        let read_clip = match clip_buffer.read().ok() {
                            Some(clip) => clip,
                            None => continue,
                        };

                        let mime_data = match &*read_clip {
                            Some(d) => d,
                            None => continue,
                        };

                        let target_atom =
                            x11::get_intern_atom_from_mime(&context.connection, mime_data.mime())?;

                        if event.property == x11rb::NONE {
                            context.reply_none(event)?;
                        } else if event.target == targets_atom {
                            context.reply_targets(event, targets_atom, target_atom)?;
                        } else if mime_data.len() < max_length - 24 {
                            context.reply_data(event, target_atom, mime_data.bytes())?;
                        } else {
                            context.reply_incr_initial(event, &mut transfer_states)?;
                        }
                    }
                    Event::PropertyNotify(event) => {
                        if event.state != xproto::Property::DELETE {
                            continue;
                        };

                        let requestor = event.atom;
                        let is_end = {
                            let state = match transfer_states.get_mut(&requestor) {
                                Some(state) => state,
                                None => continue,
                            };

                            let read_clip = match clip_buffer.read().ok() {
                                Some(clip) => clip,
                                None => continue,
                            };

                            let mime_data = match &*read_clip {
                                Some(d) => d,
                                None => continue,
                            };

                            let target_atom = x11::get_intern_atom_from_mime(
                                &context.connection,
                                mime_data.mime(),
                            )?;

                            let len = std::cmp::min(INCR_CHUNK_SIZE, mime_data.len() - state.pos);
                            context
                                .connection
                                .change_property8(
                                    xproto::PropMode::REPLACE,
                                    state.requestor,
                                    state.property,
                                    target_atom,
                                    &mime_data.bytes()[state.pos..][..len],
                                )
                                .context(error::ChangeProperty)?;

                            state.pos += len;
                            len == 0
                        };

                        if is_end {
                            transfer_states.remove(&requestor);
                        }
                        context.flush_connection()?;
                    }
                    _ => continue,
                }
            }
            Ok(())
        });

        ProviderThread(Some(handle))
    }
}

impl Context {
    #[inline]
    fn send_selection_notify_event(
        &self,
        event: xproto::SelectionRequestEvent,
    ) -> Result<(), Error> {
        let notify_event = xproto::SelectionNotifyEvent {
            response_type: xproto::SELECTION_NOTIFY_EVENT,
            sequence: event.sequence,
            time: event.time,
            requestor: event.requestor,
            selection: event.selection,
            target: event.target,
            property: event.property,
        };

        self.connection
            .send_event(true, event.requestor, x11rb::NONE, notify_event)
            .context(error::SendEvent)?;

        self.flush_connection()?;
        Ok(())
    }

    #[inline]
    fn reply_data(
        &self,
        event: xproto::SelectionRequestEvent,
        target_atom: xproto::Atom,
        data: &[u8],
    ) -> Result<(), Error> {
        self.connection
            .change_property8(
                xproto::PropMode::REPLACE,
                event.requestor,
                event.property,
                target_atom,
                data,
            )
            .context(error::ChangeProperty)?;

        self.send_selection_notify_event(event)?;
        Ok(())
    }

    #[inline]
    fn reply_targets(
        &self,
        event: xproto::SelectionRequestEvent,
        targets_atom: xproto::Atom,
        target_atom: xproto::Atom,
    ) -> Result<(), Error> {
        let data = &[targets_atom, target_atom];
        self.connection
            .change_property32(
                xproto::PropMode::REPLACE,
                event.requestor,
                event.property,
                xproto::AtomEnum::ATOM,
                data,
            )
            .context(error::ChangeProperty)?;

        self.send_selection_notify_event(event)?;
        Ok(())
    }

    #[inline]
    fn reply_incr_initial(
        &self,
        event: xproto::SelectionRequestEvent,
        transfer_states: &mut HashMap<xproto::Atom, IncrState>,
    ) -> Result<(), Error> {
        self.connection
            .change_window_attributes(
                event.requestor,
                &xproto::ChangeWindowAttributesAux::new()
                    .event_mask(xproto::EventMask::PROPERTY_CHANGE),
            )
            .context(error::ChangeWindowAttributes)?;
        self.connection
            .change_property32(
                xproto::PropMode::REPLACE,
                event.requestor,
                event.property,
                self.atom_cache.incr,
                &[0u32; 0],
            )
            .context(error::ChangeProperty)?;

        transfer_states.insert(event.property, IncrState::new(event.requestor, event.property));

        self.send_selection_notify_event(event)?;
        Ok(())
    }

    #[inline]
    fn reply_none(&self, event: xproto::SelectionRequestEvent) -> Result<(), Error> {
        self.send_selection_notify_event(event)?;
        Ok(())
    }

    #[inline]
    fn claim_selection_ownership(&self) -> Result<(), Error> {
        tracing::info!("Claim ownership of the clipboard");
        self.connection
            .set_selection_owner(self.window, self.clipboard_type(), x11rb::CURRENT_TIME)
            .context(error::ClaimSelectionOwner)?;
        self.flush_connection()?;

        if !self.check_selection_ownership()? {
            return Err(Error::MatchSelectionOwner);
        }

        Ok(())
    }

    #[inline]
    fn check_selection_ownership(&self) -> Result<bool, Error> {
        // check the ownership of clipboard again
        let selection_owner = self
            .connection
            .get_selection_owner(self.clipboard_type())
            .context(error::GetSelectionOwner)?
            .reply()
            .context(error::Reply)?
            .owner;

        Ok(selection_owner == self.window)
    }

    fn release_selection_ownership(&self) -> Result<(), Error> {
        if self.check_selection_ownership()? {
            // release ownership of the clipboard
            // x11rb::NONE means the selection will not have owner
            self.connection
                .set_selection_owner(x11rb::NONE, self.clipboard_type(), x11rb::CURRENT_TIME)
                .context(error::ReleaseSelectionOwner)?;
            self.flush_connection()?;
        }

        Ok(())
    }
}
