pub mod state_store;
pub mod ui;

use std::{future::Future, pin::Pin};

use futures::FutureExt;
use sigfinn::{ExitStatus, LifecycleManager, Shutdown};
use tokio::sync::mpsc;

use self::{
    state_store::{Action, State, StateStore},
    ui::UiManager,
};
use crate::Error;

pub async fn run(server_endpoint: http::Uri, access_token: Option<String>) -> Result<(), Error> {
    let (ui_manager, action_rx) = UiManager::new();
    let (state_store, state_rx) = StateStore::new(server_endpoint, access_token);

    let lifecycle_manager = LifecycleManager::<Error>::new();
    let handle = lifecycle_manager.handle();
    let _handle = lifecycle_manager
        .spawn("UI Manager", create_ui_manager_future(ui_manager, state_rx))
        .spawn("State Store", create_state_store_future(state_store, action_rx, handle));

    if let Ok(Err(err)) = lifecycle_manager.serve().await {
        tracing::error!("{err}");
        Err(err)
    } else {
        Ok(())
    }
}

fn create_ui_manager_future(
    ui_manager: UiManager,
    state_rx: mpsc::UnboundedReceiver<State>,
) -> impl FnOnce(Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>> {
    move |shutdown_signal| {
        async move {
            let result = ui_manager.serve(state_rx, shutdown_signal).await;
            match result {
                Ok(()) => {
                    tracing::info!("Stopped UI Manager gracefully");
                    ExitStatus::Success
                }
                Err(err) => ExitStatus::Error(Error::from(err)),
            }
        }
        .boxed()
    }
}

fn create_state_store_future(
    state_store: StateStore,
    action_rx: mpsc::UnboundedReceiver<Action>,
    handle: sigfinn::Handle<Error>,
) -> impl FnOnce(Shutdown) -> Pin<Box<dyn Future<Output = ExitStatus<Error>> + Send>> {
    move |shutdown_signal| {
        async move {
            let result = state_store.serve(action_rx, handle, shutdown_signal).await;
            match result {
                Ok(()) => {
                    tracing::info!("Stopped State Store gracefully");
                    ExitStatus::Success
                }
                Err(err) => ExitStatus::Error(Error::from(err)),
            }
        }
        .boxed()
    }
}
