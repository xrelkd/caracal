use async_trait::async_trait;
use caracal_base::model;
use caracal_proto as proto;
use tonic::Request;
use uuid::Uuid;

use crate::{error::AddUriError, Client};

#[async_trait]
pub trait Task {
    async fn add_uri(
        &self,
        create_task: model::CreateTask,
        start_immediately: bool,
    ) -> Result<Uuid, AddUriError>;
}

#[async_trait]
impl Task for Client {
    async fn add_uri(
        &self,
        model::CreateTask {
            uri,
            filename,
            output_directory,
            concurrent_number,
            connection_timeout,
            priority,
            ..
        }: model::CreateTask,
        start_immediately: bool,
    ) -> Result<Uuid, AddUriError> {
        let proto::AddUriResponse { task_id } =
            proto::TaskClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .add_uri(Request::new(proto::AddUriRequest {
                    uri: uri.to_string(),
                    start_immediately,
                    concurrent_number,
                    connection_timeout: connection_timeout.map(|t| t.as_secs()),
                    filename: filename.map(|f| f.to_string_lossy().to_string()),
                    output_directory: Some(output_directory.to_string_lossy().to_string()),
                    priority: Some(i32::from(proto::Priority::from(priority))),
                }))
                .await
                .map_err(|source| AddUriError::Status { source })?
                .into_inner();

        Uuid::try_from(task_id.ok_or(AddUriError::InvalidResponse)?)
            .map_err(|_| AddUriError::InvalidResponse)
    }
}
