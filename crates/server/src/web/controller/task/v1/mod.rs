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

#[utoipa::path(
    post,
    path = "/api/v1/task",
    request_body = model::CreateTask,
    responses(
        (status = 201, description = "Task created successfully", body = u64, example = 1),
        (status = 400, description = "Bad request", body = CreateTaskError),
        (status = 500, description = "Internal server error", body = CreateTaskError)
    ),
    tag = "Task"
)]
pub async fn create(
    Extension(task_scheduler): Extension<TaskScheduler>,
    Json(new_task): Json<model::CreateTask>,
) -> Result<(StatusCode, Json<u64>), CreateTaskError> {
    task_scheduler
        .add_uri(new_task, true)
        .await
        .map_or(Err(CreateTaskError::Internal), |n| Ok((StatusCode::CREATED, Json(n))))
}

#[utoipa::path(
    get,
    path = "/api/v1/task/{task_id}",
    params(
        ("task_id", Path, description = "ID of the task to retrieve")
    ),
    responses(
        (status = 200, description = "Task found successfully", body = model::TaskStatus),
        (status = 404, description = "Task not found", body = GetTaskError),
        (status = 500, description = "Internal server error", body = GetTaskError)
    ),
    tag = "Task"
)]
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

#[utoipa::path(
    get,
    path = "/api/v1/task",
    responses(
        (status = 200, description = "List of tasks retrieved successfully", body = Vec<model::TaskStatus>),
        (status = 500, description = "Internal server error", body = GetAllTaskStatusesError)
    ),
    tag = "Task"
)]
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

#[utoipa::path(
    delete,
    path = "/api/v1/task/remove/{task_id}",
    params(
        ("task_id", Path, description = "ID of the task to remove")
    ),
    responses(
        (status = 200, description = "Task removed successfully", body = u64),
        (status = 404, description = "Task not found", body = RemoveTaskError),
        (status = 500, description = "Internal server error", body = RemoveTaskError)
    ),
    tag = "Task"
)]
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

#[utoipa::path(
    post,
    path = "/api/v1/task/pause/{task_id}",
    params(
        ("task_id", Path, description = "ID of the task to pause")
    ),
    responses(
        (status = 200, description = "Task paused successfully", body = u64),
        (status = 404, description = "Task not found", body = RemoveTaskError),
        (status = 500, description = "Internal server error", body = RemoveTaskError)
    ),
    tag = "Task"
)]
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

#[utoipa::path(
    post,
    path = "/api/v1/task/pause/",
    responses(
        (status = 200, description = "Task paused successfully", body = u64),
        (status = 500, description = "Internal server error", body = RemoveTaskError)
    ),
    tag = "Task"
)]
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

#[utoipa::path(
    post,
    path = "/api/v1/task/resume/{task_id}",
    params(
        ("task_id", Path, description = "ID of the task to resume")
    ),
    responses(
        (status = 200, description = "Task resumed successfully", body = u64),
        (status = 404, description = "Task not found", body = RemoveTaskError),
        (status = 500, description = "Internal server error", body = RemoveTaskError)
    ),
    tag = "Task"
)]
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

#[utoipa::path(
    post,
    path = "/api/v1/task/resume/",
    responses(
        (status = 200, description = "Task resumed successfully", body = u64),
        (status = 500, description = "Internal server error", body = RemoveTaskError)
    ),
    tag = "Task"
)]
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
