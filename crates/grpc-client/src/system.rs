use caracal_proto as proto;
use tonic::Request;

use crate::{error::GetSystemVersionError, Client};

pub trait System {
    async fn get_version(&self) -> Result<semver::Version, GetSystemVersionError>;
}

impl System for Client {
    async fn get_version(&self) -> Result<semver::Version, GetSystemVersionError> {
        let proto::GetSystemVersionResponse { major, minor, patch } =
            proto::SystemClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .get_version(Request::new(()))
                .await
                .map_err(|source| GetSystemVersionError::Status { source })?
                .into_inner();
        Ok(semver::Version {
            major,
            minor,
            patch,
            pre: semver::Prerelease::EMPTY,
            build: semver::BuildMetadata::EMPTY,
        })
    }
}
