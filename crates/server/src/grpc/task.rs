use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use caracal_base::model;
use caracal_engine::{TaskScheduler, TaskStatus};
use caracal_proto as proto;
use time::OffsetDateTime;
use uuid::Uuid;

pub struct TaskService {
    task_scheduler: TaskScheduler,

    default_output_directory: PathBuf,
}

impl TaskService {
    #[inline]
    pub fn new<P>(task_scheduler: TaskScheduler, default_output_directory: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            task_scheduler,
            default_output_directory: default_output_directory.as_ref().to_path_buf(),
        }
    }
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
            directory_path: output_directory
                .map_or_else(|| self.default_output_directory.clone(), PathBuf::from),
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

        Ok(tonic::Response::new(proto::AddUriResponse {
            task_id: Some(proto::Uuid::from(task_id)),
        }))
    }

    async fn pause(
        &self,
        request: tonic::Request<proto::PauseTaskRequest>,
    ) -> Result<tonic::Response<proto::PauseTaskResponse>, tonic::Status> {
        let proto::PauseTaskRequest { task_id } = request.into_inner();

        let task_id = Uuid::try_from(task_id.ok_or_else(task_id_missing_status)?)
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

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
            .map(|task_ids| {
                let task_ids = task_ids.iter().copied().map(proto::Uuid::from).collect();
                tonic::Response::new(proto::PauseAllTasksResponse { task_ids })
            })
            .map_err(service_shutdown_status)
    }

    async fn resume(
        &self,
        request: tonic::Request<proto::ResumeTaskRequest>,
    ) -> Result<tonic::Response<proto::ResumeTaskResponse>, tonic::Status> {
        let proto::ResumeTaskRequest { task_id } = request.into_inner();

        let task_id = Uuid::try_from(task_id.ok_or_else(task_id_missing_status)?)
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

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
            .map(|task_ids| {
                let task_ids = task_ids.iter().copied().map(proto::Uuid::from).collect();
                tonic::Response::new(proto::ResumeAllTasksResponse { task_ids })
            })
            .map_err(service_shutdown_status)
    }

    async fn remove(
        &self,
        request: tonic::Request<proto::RemoveTaskRequest>,
    ) -> Result<tonic::Response<proto::RemoveTaskResponse>, tonic::Status> {
        let proto::RemoveTaskRequest { task_id } = request.into_inner();

        let task_id = Uuid::try_from(task_id.ok_or_else(task_id_missing_status)?)
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

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

        let task_id = Uuid::try_from(task_id.ok_or_else(task_id_missing_status)?)
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

        if let Some(TaskStatus { id, status, state, priority, creation_timestamp }) =
            self.task_scheduler.get_task_status(task_id).await.map_err(service_shutdown_status)?
        {
            let chunks = status
                .chunks()
                .iter()
                .map(|chunk| proto::Chunk {
                    start: chunk.start,
                    end: chunk.end,
                    received: chunk.received,
                })
                .collect();
            Ok(tonic::Response::new(proto::GetTaskStatusResponse {
                metadata: Some(proto::TaskMetadata {
                    id: Some(proto::Uuid::from(id)),
                    priority: i32::from(proto::Priority::from(priority)),
                    creation_timestamp: Some(proto::datetime_to_timestamp(&creation_timestamp)),
                    size: Some(status.content_length()),
                    file_path: status.file_path().to_str().unwrap_or_default().to_string(),
                }),
                state: i32::from(proto::TaskState::from(state)),
                received_bytes: status.total_received(),
                total_length: status.content_length(),
                chunks,
            }))
        } else {
            Err(tonic::Status::not_found(task_id.to_string()))
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn service_shutdown_status<E>(_err: E) -> tonic::Status {
    tonic::Status::unavailable("Caracal is shutting down")
}

fn task_id_missing_status() -> tonic::Status {
    tonic::Status::invalid_argument("task_id is missing")
}
