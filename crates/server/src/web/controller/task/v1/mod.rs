mod error;

use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
};
use caracal_base::model;
use caracal_engine::TaskScheduler;

use self::error::{
    CreateTaskError, GetAllTaskStatusesError, GetTaskError, PauseAllTasksError, PauseTaskError,
    RemoveTaskError, ResumeAllTasksError, ResumeTaskStatusesError,
};

pub async fn create(
    Extension(task_scheduler): Extension<TaskScheduler>,
    Json(new_task): Json<model::CreateTask>,
) -> Result<(StatusCode, Json<u64>), CreateTaskError> {
    task_scheduler
        .add_uri(new_task, true)
        .await
        .map_or(Err(CreateTaskError::Internal), |n| Ok((StatusCode::CREATED, Json(n))))
}

pub async fn get(
    Extension(task_scheduler): Extension<TaskScheduler>,
    Path(task_id): Path<u64>,
) -> Result<(StatusCode, Json<model::TaskStatus>), GetTaskError> {
    match task_scheduler.get_task_status(task_id).await {
        Ok(Some(task_status)) => Ok((StatusCode::OK, Json(task_status))),
        Ok(None) => Err(GetTaskError::NotFound),
        Err(source) => {
            tracing::error!("{source}");
            Err(GetTaskError::Internal)
        }
    }
}

pub async fn list(
    Extension(task_scheduler): Extension<TaskScheduler>,
) -> Result<(StatusCode, Json<Vec<model::TaskStatus>>), GetAllTaskStatusesError> {
    match task_scheduler.get_all_task_statuses().await {
        Ok(tasks) => Ok((StatusCode::OK, Json(tasks))),
        Err(source) => {
            tracing::error!("{source}");
            Err(GetAllTaskStatusesError::Internal)
        }
    }
}

pub async fn remove(
    Extension(task_scheduler): Extension<TaskScheduler>,
    Path(task_id): Path<u64>,
) -> Result<(StatusCode, Json<u64>), RemoveTaskError> {
    match task_scheduler.remove_task(task_id).await {
        Ok(Some(task_id)) => Ok((StatusCode::OK, Json(task_id))),
        Ok(None) => Err(RemoveTaskError::NotFound),
        Err(source) => {
            tracing::error!("{source}");
            Err(RemoveTaskError::Internal)
        }
    }
}

pub async fn pause(
    Extension(task_scheduler): Extension<TaskScheduler>,
    Path(task_id): Path<u64>,
) -> Result<(StatusCode, Json<u64>), PauseTaskError> {
    match task_scheduler.pause_task(task_id).await {
        Ok(Some(task_id)) => Ok((StatusCode::OK, Json(task_id))),
        Ok(None) => Err(PauseTaskError::NotFound),
        Err(source) => {
            tracing::error!("{source}");
            Err(PauseTaskError::Internal)
        }
    }
}

pub async fn pause_all(
    Extension(task_scheduler): Extension<TaskScheduler>,
) -> Result<StatusCode, PauseAllTasksError> {
    match task_scheduler.pause_all_tasks() {
        Ok(()) => Ok(StatusCode::OK),
        Err(source) => {
            tracing::error!("{source}");
            Err(PauseAllTasksError::Internal)
        }
    }
}

pub async fn resume(
    Extension(task_scheduler): Extension<TaskScheduler>,
    Path(task_id): Path<u64>,
) -> Result<(StatusCode, Json<u64>), ResumeTaskStatusesError> {
    match task_scheduler.resume_task(task_id).await {
        Ok(Some(task_id)) => Ok((StatusCode::OK, Json(task_id))),
        Ok(None) => Err(ResumeTaskStatusesError::NotFound),
        Err(source) => {
            tracing::error!("{source}");
            Err(ResumeTaskStatusesError::Internal)
        }
    }
}

pub async fn resume_all(
    Extension(task_scheduler): Extension<TaskScheduler>,
) -> Result<StatusCode, ResumeAllTasksError> {
    match task_scheduler.resume_all_tasks() {
        Ok(()) => Ok(StatusCode::OK),
        Err(source) => {
            tracing::error!("{source}");
            Err(ResumeAllTasksError::Internal)
        }
    }
}
