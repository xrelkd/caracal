use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not create tokio runtime, error: {source}"))]
    InitializeTokioRuntime { source: std::io::Error },

    #[snafu(display("Could not get current directory, error: {source}"))]
    GetCurrentDirectory { source: std::io::Error },

    #[snafu(display("Error occurs while running lifecycle manager, error: {source}"))]
    LifecycleManager { source: sigfinn::Error },

    #[snafu(display("{source}"))]
    Config {
        #[snafu(source(from(crate::config::Error, Box::new)))]
        source: Box<crate::config::Error>,
    },

    #[snafu(display("{source}"))]
    Client { source: caracal_grpc_client::Error },

    #[snafu(display("No URI is provided"))]
    NoUri,

    #[snafu(display("{source}"))]
    Ui { source: crate::tui::ui::Error },

    #[snafu(display("{source}"))]
    StateStore { source: crate::tui::state_store::Error },
}

impl From<crate::config::Error> for Error {
    fn from(source: crate::config::Error) -> Self { Self::Config { source: Box::new(source) } }
}

impl From<caracal_grpc_client::Error> for Error {
    fn from(source: caracal_grpc_client::Error) -> Self { Self::Client { source } }
}

impl From<crate::tui::ui::Error> for Error {
    fn from(source: crate::tui::ui::Error) -> Self { Self::Ui { source } }
}

impl From<crate::tui::state_store::Error> for Error {
    fn from(source: crate::tui::state_store::Error) -> Self { Self::StateStore { source } }
}
