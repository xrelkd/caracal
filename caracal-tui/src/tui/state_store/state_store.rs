use std::time::Duration;

use caracal_grpc_client as grpc;
use futures::FutureExt;
use grpc::{System, Task};
use snafu::OptionExt;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use crate::tui::state_store::{error, Action, Error, State};

#[derive(Debug)]
pub struct StateStore {
    state_tx: mpsc::UnboundedSender<State>,

    server_endpoint: http::Uri,

    access_token: Option<String>,
}

impl StateStore {
    pub fn new(
        server_endpoint: http::Uri,
        access_token: Option<String>,
    ) -> (Self, mpsc::UnboundedReceiver<State>) {
        let (state_tx, state_rx) = mpsc::unbounded_channel();
        (Self { state_tx, server_endpoint, access_token }, state_rx)
    }

    pub async fn serve(
        self,
        mut action_rx: mpsc::UnboundedReceiver<Action>,
        lifecycle_manager_handle: sigfinn::Handle<crate::Error>,
        shutdown: sigfinn::Shutdown,
    ) -> Result<(), Error> {
        let mut state = State::default();
        state.set_server_endpoint(self.server_endpoint);
        state.set_access_token(self.access_token);

        let mut ticker = tokio::time::interval(Duration::from_millis(500));
        let mut shutdown = shutdown.into_stream();

        // The initial state once
        self.state_tx.send(state.clone()).ok().context(error::StateReceiverClosedSnafu)?;

        let mut client =
            grpc::Client::new(state.server_endpoint().clone(), state.access_token()).await.ok();

        loop {
            let action = tokio::select! {
                Some(action) = action_rx.recv() => action,
                // Tick to terminate the select every N milliseconds
               _ = ticker.tick() => Action::GetAllTaskStatuses,
                // Catch and handle interrupt signal to gracefully shutdown
                Some(()) = shutdown.next() => {
                    break;
                }
            };

            match action {
                Action::Shutdown => {
                    lifecycle_manager_handle.shutdown();
                    break;
                }
                Action::GetAllTaskStatuses => {
                    state.tick_timer();
                    if let Some(ref client) = client {
                        if let Ok(mut task_statuses) = client.get_all_task_statuses().await {
                            task_statuses.sort_unstable_by_key(|status| status.id);
                            state.set_task_statuses(task_statuses);
                        }
                        if let Ok(version) = client.get_version().await {
                            state.set_daemon_version(version);
                        } else {
                            state.mark_disconnected();
                        }
                    } else {
                        state.mark_disconnected();
                        client = grpc::Client::new(
                            state.server_endpoint().clone(),
                            state.access_token(),
                        )
                        .await
                        .map_err(|err| tracing::warn!("{err}"))
                        .ok();
                    }
                }
                Action::IncreaseConcurrentNumber { task_id } => {
                    if let Some(ref client) = client {
                        if let Err(err) = client.increase_concurrent_number(task_id).await {
                            tracing::warn!("{err}");
                        }
                    }
                }
                Action::DecreaseConcurrentNumber { task_id } => {
                    if let Some(ref client) = client {
                        if let Err(err) = client.decrease_concurrent_number(task_id).await {
                            tracing::warn!("{err}");
                        }
                    }
                }
                Action::RemoveTask { task_id } => {
                    if let Some(ref client) = client {
                        if let Err(err) = client.remove(task_id).await {
                            tracing::warn!("{err}");
                        }
                    }
                }
                Action::PauseTask { task_id } => {
                    if let Some(ref client) = client {
                        if let Err(err) = client.pause(task_id).await {
                            tracing::warn!("{err}");
                        }
                    }
                }
                Action::ResumeTask { task_id } => {
                    if let Some(ref client) = client {
                        if let Err(err) = client.resume(task_id).await {
                            tracing::warn!("{err}");
                        }
                    }
                }
                _ => continue,
            }
            self.state_tx.send(state.clone()).ok().context(error::StateReceiverClosedSnafu)?;
        }
        self.state_tx.send(state.clone()).ok().context(error::StateReceiverClosedSnafu)?;

        Ok(())
    }
}
