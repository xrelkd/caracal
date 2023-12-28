#![allow(clippy::module_name_repetitions)]

use std::{fmt, path::PathBuf};

use snafu::{Backtrace, Snafu};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display(
        "Error occurs while connecting to Caracal Server gRPC endpoint `{endpoint}` via HTTP, \
         error: {source}"
    ))]
    ConnectToServerViaHttp {
        endpoint: http::Uri,
        source: tonic::transport::Error,
        backtrace: Backtrace,
    },

    #[snafu(display(
        "Error occurs while connecting to Caracal Server gRPC endpoint `{}` via local \
         socket, error: {source}",
        socket.display()
    ))]
    ConnectToServerViaLocalSocket {
        socket: PathBuf,
        source: tonic::transport::Error,
        backtrace: Backtrace,
    },
}

#[derive(Debug)]
pub enum AddUriError {
    Status { source: tonic::Status },
}

impl fmt::Display for AddUriError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}

#[derive(Debug)]
pub enum GetSystemVersionError {
    Status { source: tonic::Status },
}

impl fmt::Display for GetSystemVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { source } => source.fmt(f),
        }
    }
}
