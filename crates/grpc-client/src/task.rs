use async_trait::async_trait;
use caracal_base::model;
use caracal_proto as proto;
use tonic::Request;
use uuid::Uuid;

use crate::{
    error::{AddUriError, PauseTaskError, RemoveTaskError, ResumeAllTasksError, ResumeTaskError},
    Client,
};

#[async_trait]
pub trait Task {
    async fn add_uri(
        &self,
        create_task: model::CreateTask,
        start_immediately: bool,
    ) -> Result<Uuid, AddUriError>;

    async fn pause(&self, task_id: Uuid) -> Result<bool, PauseTaskError>;

    async fn resume(&self, task_id: Uuid) -> Result<bool, ResumeTaskError>;

    async fn remove(&self, task_id: Uuid) -> Result<bool, RemoveTaskError>;

    async fn resume_all(&self) -> Result<Vec<Uuid>, ResumeAllTasksError>;
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

    async fn pause(&self, task_id: Uuid) -> Result<bool, PauseTaskError> {
        let proto::PauseTaskResponse { ok } =
            proto::TaskClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .pause(Request::new(proto::PauseTaskRequest {
                    task_id: Some(proto::Uuid::from(task_id)),
                }))
                .await
                .map_err(|source| PauseTaskError::Status { source })?
                .into_inner();
        Ok(ok)
    }

    async fn resume(&self, task_id: Uuid) -> Result<bool, ResumeTaskError> {
        let proto::ResumeTaskResponse { ok } =
            proto::TaskClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .resume(Request::new(proto::ResumeTaskRequest {
                    task_id: Some(proto::Uuid::from(task_id)),
                }))
                .await
                .map_err(|source| ResumeTaskError::Status { source })?
                .into_inner();
        Ok(ok)
    }

    async fn remove(&self, task_id: Uuid) -> Result<bool, RemoveTaskError> {
        let proto::RemoveTaskResponse { ok } =
            proto::TaskClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .remove(Request::new(proto::RemoveTaskRequest {
                    task_id: Some(proto::Uuid::from(task_id)),
                }))
                .await
                .map_err(|source| RemoveTaskError::Status { source })?
                .into_inner();
        Ok(ok)
    }

    async fn resume_all(&self) -> Result<Vec<Uuid>, ResumeAllTasksError> {
        let proto::ResumeAllTasksResponse { task_ids } =
            proto::TaskClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .resume_all(Request::new(()))
                .await
                .map_err(|source| ResumeAllTasksError::Status { source })?
                .into_inner();

        let task_ids = {
            let mut ret = Vec::with_capacity(task_ids.len());
            for task_id in task_ids {
                ret.push(
                    Uuid::try_from(task_id).map_err(|_| ResumeAllTasksError::InvalidResponse)?,
                );
            }
            ret
        };

        Ok(task_ids)
    }
}
