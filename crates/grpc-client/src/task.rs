use std::path::PathBuf;

use async_trait::async_trait;
use caracal_base::model;
use caracal_proto as proto;
use tonic::Request;

use crate::{
    error::{
        AddUriError, GetAllTaskStatusesError, GetTaskStatusError, PauseAllTasksError,
        PauseTaskError, RemoveTaskError, ResumeAllTasksError, ResumeTaskError,
    },
    Client,
};

#[async_trait]
pub trait Task {
    async fn add_uri(
        &self,
        create_task: model::CreateTask,
        start_immediately: bool,
    ) -> Result<u64, AddUriError>;

    async fn pause(&self, task_id: u64) -> Result<bool, PauseTaskError>;

    async fn resume(&self, task_id: u64) -> Result<bool, ResumeTaskError>;

    async fn remove(&self, task_id: u64) -> Result<bool, RemoveTaskError>;

    async fn pause_all(&self) -> Result<Vec<u64>, PauseAllTasksError>;

    async fn resume_all(&self) -> Result<Vec<u64>, ResumeAllTasksError>;

    async fn get_task_status(&self, task_id: u64) -> Result<model::TaskStatus, GetTaskStatusError>;

    async fn get_all_task_statuses(
        &self,
    ) -> Result<Vec<model::TaskStatus>, GetAllTaskStatusesError>;
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
    ) -> Result<u64, AddUriError> {
        let proto::AddUriResponse { task_id } =
            proto::TaskClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .add_uri(Request::new(proto::AddUriRequest {
                    uri: uri.to_string(),
                    start_immediately,
                    concurrent_number,
                    connection_timeout: connection_timeout.map(|t| t.as_secs()),
                    filename: filename.map(|f| f.to_string_lossy().to_string()),
                    output_directory: output_directory
                        .map(|path| path.to_string_lossy().to_string()),
                    priority: Some(i32::from(proto::Priority::from(priority))),
                }))
                .await
                .map_err(|source| AddUriError::Status { source })?
                .into_inner();

        Ok(task_id)
    }

    async fn pause(&self, task_id: u64) -> Result<bool, PauseTaskError> {
        let proto::PauseTaskResponse { ok } =
            proto::TaskClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .pause(Request::new(proto::PauseTaskRequest { task_id }))
                .await
                .map_err(|source| PauseTaskError::Status { source })?
                .into_inner();
        Ok(ok)
    }

    async fn resume(&self, task_id: u64) -> Result<bool, ResumeTaskError> {
        let proto::ResumeTaskResponse { ok } =
            proto::TaskClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .resume(Request::new(proto::ResumeTaskRequest { task_id }))
                .await
                .map_err(|source| ResumeTaskError::Status { source })?
                .into_inner();
        Ok(ok)
    }

    async fn remove(&self, task_id: u64) -> Result<bool, RemoveTaskError> {
        let proto::RemoveTaskResponse { ok } =
            proto::TaskClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .remove(Request::new(proto::RemoveTaskRequest { task_id }))
                .await
                .map_err(|source| RemoveTaskError::Status { source })?
                .into_inner();
        Ok(ok)
    }

    async fn pause_all(&self) -> Result<Vec<u64>, PauseAllTasksError> {
        let proto::PauseAllTasksResponse { task_ids } =
            proto::TaskClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .pause_all(Request::new(()))
                .await
                .map_err(|source| PauseAllTasksError::Status { source })?
                .into_inner();

        Ok(task_ids)
    }

    async fn resume_all(&self) -> Result<Vec<u64>, ResumeAllTasksError> {
        let proto::ResumeAllTasksResponse { task_ids } =
            proto::TaskClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .resume_all(Request::new(()))
                .await
                .map_err(|source| ResumeAllTasksError::Status { source })?
                .into_inner();

        Ok(task_ids)
    }

    async fn get_task_status(&self, task_id: u64) -> Result<model::TaskStatus, GetTaskStatusError> {
        let proto::GetTaskStatusResponse { status } =
            proto::TaskClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .get_task_status(Request::new(proto::GetTaskStatusRequest { task_id }))
                .await
                .map_err(|source| GetTaskStatusError::Status { source })?
                .into_inner();
        let proto::TaskStatus { metadata, state, total_length, chunks, concurrent_number, .. } =
            status.ok_or(GetTaskStatusError::InvalidResponse)?;
        let proto::TaskMetadata { id, file_path, priority, creation_timestamp, .. } =
            metadata.ok_or(GetTaskStatusError::InvalidResponse)?;
        let creation_timestamp = creation_timestamp.ok_or(GetTaskStatusError::InvalidResponse)?;

        Ok(model::TaskStatus {
            id,
            file_path: PathBuf::from(file_path),
            content_length: total_length,
            chunks: chunks.into_iter().map(model::ProgressChunk::from).collect(),
            concurrent_number: usize::try_from(concurrent_number).unwrap_or(1),
            state: model::TaskState::from(
                proto::TaskState::try_from(state)
                    .map_err(|_| GetTaskStatusError::InvalidResponse)?,
            ),
            priority: priority.into(),
            creation_timestamp: proto::timestamp_to_datetime(&creation_timestamp)
                .map_err(|_| GetTaskStatusError::InvalidResponse)?,
        })
    }

    async fn get_all_task_statuses(
        &self,
    ) -> Result<Vec<model::TaskStatus>, GetAllTaskStatusesError> {
        let proto::GetAllTaskStatusesResponse { statuses } =
            proto::TaskClient::with_interceptor(self.channel.clone(), self.interceptor.clone())
                .get_all_task_statuses(Request::new(()))
                .await
                .map_err(|source| GetAllTaskStatusesError::Status { source })?
                .into_inner();

        let mut ret = Vec::with_capacity(statuses.len());
        for proto::TaskStatus {
            metadata, state, total_length, chunks, concurrent_number, ..
        } in statuses
        {
            let proto::TaskMetadata { id, file_path, priority, creation_timestamp, .. } =
                metadata.ok_or(GetAllTaskStatusesError::InvalidResponse)?;
            let creation_timestamp = proto::timestamp_to_datetime(
                &creation_timestamp.ok_or(GetAllTaskStatusesError::InvalidResponse)?,
            )
            .map_err(|_| GetAllTaskStatusesError::InvalidResponse)?;
            let state = model::TaskState::from(
                proto::TaskState::try_from(state)
                    .map_err(|_| GetAllTaskStatusesError::InvalidResponse)?,
            );
            ret.push(model::TaskStatus {
                id,
                file_path: PathBuf::from(file_path),
                content_length: total_length,
                chunks: chunks.into_iter().map(model::ProgressChunk::from).collect(),
                concurrent_number: usize::try_from(concurrent_number).unwrap_or(1),
                state,
                priority: model::Priority::from(priority),
                creation_timestamp,
            });
        }

        Ok(ret)
    }
}
