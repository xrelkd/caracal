use std::{path::PathBuf, time::Duration};

use caracal_base::model;
use caracal_engine::TaskScheduler;
use caracal_proto as proto;
use time::OffsetDateTime;

pub struct TaskService {
    task_scheduler: TaskScheduler,
}

impl TaskService {
    #[inline]
    pub const fn new(task_scheduler: TaskScheduler) -> Self { Self { task_scheduler } }
}

#[tonic::async_trait]
impl proto::Task for TaskService {
    async fn add_uri(
        &self,
        request: tonic::Request<proto::AddUriRequest>,
    ) -> Result<tonic::Response<proto::AddUriResponse>, tonic::Status> {
        let proto::AddUriRequest {
            uri,
            output_directory,
            filename,
            priority,
            start_immediately,
            connection_timeout,
            concurrent_number,
        } = request.into_inner();
        let uri = uri
            .parse::<http::Uri>()
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;
        let new_task = model::CreateTask {
            uri,
            filename: filename.map(PathBuf::from),
            output_directory: output_directory.map(PathBuf::from),
            concurrent_number,
            connection_timeout: connection_timeout.map(Duration::from_secs),
            priority: priority.map_or(model::Priority::Normal, |v| {
                model::Priority::from(
                    proto::Priority::try_from(v).unwrap_or(proto::Priority::Normal),
                )
            }),
            creation_timestamp: OffsetDateTime::now_utc(),
        };

        let task_id = self
            .task_scheduler
            .add_uri(new_task, start_immediately)
            .await
            .map_err(service_shutdown_status)?;

        Ok(tonic::Response::new(proto::AddUriResponse { task_id }))
    }

    async fn pause(
        &self,
        request: tonic::Request<proto::PauseTaskRequest>,
    ) -> Result<tonic::Response<proto::PauseTaskResponse>, tonic::Status> {
        let proto::PauseTaskRequest { task_id } = request.into_inner();

        self.task_scheduler
            .pause_task(task_id)
            .await
            .map(|_| tonic::Response::new(proto::PauseTaskResponse { ok: true }))
            .map_err(service_shutdown_status)
    }

    async fn pause_all(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::PauseAllTasksResponse>, tonic::Status> {
        self.task_scheduler.pause_all_tasks().map_err(service_shutdown_status)?;
        self.task_scheduler
            .get_all_tasks()
            .await
            .map(|task_ids| tonic::Response::new(proto::PauseAllTasksResponse { task_ids }))
            .map_err(service_shutdown_status)
    }

    async fn resume(
        &self,
        request: tonic::Request<proto::ResumeTaskRequest>,
    ) -> Result<tonic::Response<proto::ResumeTaskResponse>, tonic::Status> {
        let proto::ResumeTaskRequest { task_id } = request.into_inner();

        self.task_scheduler
            .resume_task(task_id)
            .await
            .map(|_| tonic::Response::new(proto::ResumeTaskResponse { ok: true }))
            .map_err(service_shutdown_status)
    }

    async fn resume_all(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::ResumeAllTasksResponse>, tonic::Status> {
        self.task_scheduler.resume_all_tasks().map_err(service_shutdown_status)?;
        self.task_scheduler
            .get_all_tasks()
            .await
            .map(|task_ids| tonic::Response::new(proto::ResumeAllTasksResponse { task_ids }))
            .map_err(service_shutdown_status)
    }

    async fn remove(
        &self,
        request: tonic::Request<proto::RemoveTaskRequest>,
    ) -> Result<tonic::Response<proto::RemoveTaskResponse>, tonic::Status> {
        let proto::RemoveTaskRequest { task_id } = request.into_inner();

        self.task_scheduler
            .remove_task(task_id)
            .await
            .map(|_| tonic::Response::new(proto::RemoveTaskResponse { ok: true }))
            .map_err(service_shutdown_status)
    }

    async fn get_task_status(
        &self,
        request: tonic::Request<proto::GetTaskStatusRequest>,
    ) -> Result<tonic::Response<proto::GetTaskStatusResponse>, tonic::Status> {
        let proto::GetTaskStatusRequest { task_id } = request.into_inner();

        if let Some(model::TaskStatus {
            id,
            file_path,
            state,
            priority,
            creation_timestamp,
            chunks,
            content_length,
            concurrent_number,
        }) =
            self.task_scheduler.get_task_status(task_id).await.map_err(service_shutdown_status)?
        {
            let received_bytes = chunks.iter().map(|chunk| chunk.received).sum();
            let chunks = chunks.iter().cloned().map(proto::Chunk::from).collect();
            Ok(tonic::Response::new(proto::GetTaskStatusResponse {
                status: Some(proto::TaskStatus {
                    metadata: Some(proto::TaskMetadata {
                        id,
                        priority: i32::from(proto::Priority::from(priority)),
                        creation_timestamp: Some(proto::datetime_to_timestamp(&creation_timestamp)),
                        size: Some(content_length),
                        file_path: file_path.to_str().unwrap_or_default().to_string(),
                    }),
                    state: i32::from(proto::TaskState::from(state)),
                    received_bytes,
                    total_length: content_length,
                    concurrent_number: concurrent_number as u64,
                    chunks,
                }),
            }))
        } else {
            Err(tonic::Status::not_found(task_id.to_string()))
        }
    }

    async fn get_all_task_statuses(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<proto::GetAllTaskStatusesResponse>, tonic::Status> {
        let task_status =
            self.task_scheduler.get_all_task_statuses().await.map_err(service_shutdown_status)?;
        let mut task_statuses = Vec::with_capacity(task_status.len());
        for sts in task_status {
            let model::TaskStatus {
                id,
                file_path,
                state,
                priority,
                creation_timestamp,
                chunks,
                content_length,
                concurrent_number,
            } = sts;

            let received_bytes = chunks.iter().map(|chunk| chunk.received).sum();
            let chunks = chunks.iter().cloned().map(proto::Chunk::from).collect();

            task_statuses.push(proto::TaskStatus {
                metadata: Some(proto::TaskMetadata {
                    id,
                    priority: i32::from(proto::Priority::from(priority)),
                    creation_timestamp: Some(proto::datetime_to_timestamp(&creation_timestamp)),
                    size: Some(content_length),
                    file_path: file_path.to_str().unwrap_or_default().to_string(),
                }),
                state: i32::from(proto::TaskState::from(state)),
                received_bytes,
                total_length: content_length,
                concurrent_number: u64::try_from(concurrent_number).unwrap_or(1),
                chunks,
            });
        }
        Ok(tonic::Response::new(proto::GetAllTaskStatusesResponse { statuses: task_statuses }))
    }

    async fn increase_concurrent_number(
        &self,
        request: tonic::Request<proto::IncreaseConcurrentNumberRequest>,
    ) -> Result<tonic::Response<proto::IncreaseConcurrentNumberResponse>, tonic::Status> {
        let proto::IncreaseConcurrentNumberRequest { task_id } = request.into_inner();
        self.task_scheduler.increase_concurrent_number(task_id).map_err(service_shutdown_status)?;
        Ok(tonic::Response::new(proto::IncreaseConcurrentNumberResponse { ok: true }))
    }

    async fn decrease_concurrent_number(
        &self,
        request: tonic::Request<proto::DecreaseConcurrentNumberRequest>,
    ) -> Result<tonic::Response<proto::DecreaseConcurrentNumberResponse>, tonic::Status> {
        let proto::DecreaseConcurrentNumberRequest { task_id } = request.into_inner();
        self.task_scheduler.decrease_concurrent_number(task_id).map_err(service_shutdown_status)?;
        Ok(tonic::Response::new(proto::DecreaseConcurrentNumberResponse { ok: true }))
    }
}

#[allow(clippy::needless_pass_by_value)]
fn service_shutdown_status<E>(_err: E) -> tonic::Status {
    tonic::Status::unavailable("Caracal is shutting down")
}
