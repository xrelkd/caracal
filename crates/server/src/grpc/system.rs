use std::sync::LazyLock;

use caracal_proto as proto;
use tonic::{Request, Response, Status};

pub static GET_SYSTEM_VERSION_RESPONSE: LazyLock<proto::GetSystemVersionResponse> =
    LazyLock::new(|| proto::GetSystemVersionResponse {
        major: caracal_base::PROJECT_SEMVER.major,
        minor: caracal_base::PROJECT_SEMVER.minor,
        patch: caracal_base::PROJECT_SEMVER.patch,
    });

pub struct SystemService {}

impl SystemService {
    #[inline]
    pub const fn new() -> Self { Self {} }
}

#[tonic::async_trait]
impl proto::System for SystemService {
    async fn get_version(
        &self,
        _request: Request<()>,
    ) -> Result<Response<proto::GetSystemVersionResponse>, Status> {
        Ok(Response::new(*GET_SYSTEM_VERSION_RESPONSE))
    }
}
