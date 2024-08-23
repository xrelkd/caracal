pub mod error;
mod interceptor;
mod system;
mod task;

use std::fmt;

use snafu::ResultExt;
use tokio::net::UnixStream;

use self::interceptor::Interceptor;
pub use self::{
    error::{Error, Result},
    system::System,
    task::Task,
};

#[derive(Clone, Debug)]
pub struct Client {
    channel: tonic::transport::Channel,
    interceptor: Interceptor,
}

impl Client {
    /// # Errors
    pub async fn new<A>(grpc_endpoint: http::Uri, access_token: Option<A>) -> Result<Self>
    where
        A: fmt::Display + Send,
    {
        tracing::info!("Connect to server via endpoint `{grpc_endpoint}`");
        let scheme = grpc_endpoint.scheme();
        if scheme == Some(&http::uri::Scheme::HTTP) || scheme == Some(&http::uri::Scheme::HTTPS) {
            Self::connect_http(&grpc_endpoint, access_token).await
        } else {
            Self::connect_local_socket(&grpc_endpoint, access_token).await
        }
    }

    /// # Errors
    ///
    /// This function will an error if the server is not connected.
    // SAFETY: it will never panic because `grpc_endpoint` is a valid URL
    #[allow(clippy::missing_panics_doc)]
    pub async fn connect_http<A>(grpc_endpoint: &http::Uri, access_token: Option<A>) -> Result<Self>
    where
        A: fmt::Display + Send,
    {
        let interceptor = Interceptor::new(access_token);
        let channel = tonic::transport::Endpoint::from_shared(grpc_endpoint.to_string())
            .expect("`grpc_endpoint` is a valid URL; qed")
            .connect()
            .await
            .with_context(|_| error::ConnectToServerViaHttpSnafu {
                endpoint: grpc_endpoint.clone(),
            })?;
        Ok(Self { channel, interceptor })
    }

    /// # Errors
    ///
    /// This function will an error if the server is not connected.
    // SAFETY: it will never panic because `dummy_uri` is a valid URL
    #[allow(clippy::missing_panics_doc)]
    pub async fn connect_local_socket<A>(uri: &http::Uri, access_token: Option<A>) -> Result<Self>
    where
        A: fmt::Display + Send,
    {
        let socket_path = uri.path();

        let channel = tonic::transport::Endpoint::try_from(format!("file://[::]/{socket_path}"))
            .expect("`uri` is a valid URL; qed")
            .connect_with_connector(tower::service_fn(|uri: tonic::transport::Uri| async move {
                // Connect to a Uds socket
                Ok::<_, std::io::Error>(hyper_util::rt::TokioIo::new(
                    UnixStream::connect(uri.path()).await?,
                ))
            }))
            .await
            .with_context(|_| error::ConnectToServerViaLocalSocketSnafu { socket: socket_path })?;
        let interceptor = Interceptor::new(access_token);

        Ok(Self { channel, interceptor })
    }
}
