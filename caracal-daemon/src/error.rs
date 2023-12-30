use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not create tokio runtime, error: {source}"))]
    InitializeTokioRuntime { source: std::io::Error },

    #[snafu(display("{source}"))]
    Config { source: Box<crate::config::Error> },

    #[snafu(display("{source}"))]
    Application { source: Box<caracal_server::Error> },

    #[snafu(display("Error occurs while interacting with server, error: {error}"))]
    Operation { error: String },
}

impl From<crate::config::Error> for Error {
    fn from(source: crate::config::Error) -> Self { Self::Config { source: Box::new(source) } }
}

impl From<caracal_server::Error> for Error {
    fn from(source: caracal_server::Error) -> Self {
        Self::Application { source: Box::new(source) }
    }
}
