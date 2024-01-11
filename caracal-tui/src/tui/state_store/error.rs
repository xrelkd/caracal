use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("State receiver is closed"))]
    StateReceiverClosed,

    #[snafu(display("{source}"))]
    Client { source: caracal_grpc_client::Error },
}

impl From<caracal_grpc_client::Error> for Error {
    fn from(source: caracal_grpc_client::Error) -> Self { Self::Client { source } }
}
