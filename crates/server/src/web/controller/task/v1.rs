use axum::{
    body,
    extract::{Extension, Json},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use caracal_base::model;
use caracal_engine::TaskScheduler;

#[derive(Clone, Debug)]
pub enum CreateTaskError {
    Internal,
}

impl IntoResponse for CreateTaskError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, body::Body::from(String::new())),
        };

        Response::builder()
            .status(status)
            .body(body)
            .expect("response should always build successfully")
    }
}

#[derive(Clone, Debug)]
pub enum GetAllTaskStatusesError {
    Internal,
}

impl IntoResponse for GetAllTaskStatusesError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, body::Body::from(String::new())),
        };

        Response::builder()
            .status(status)
            .body(body)
            .expect("response should always build successfully")
    }
}

pub async fn create(
    Extension(task_scheduler): Extension<TaskScheduler>,
    Json(new_task): Json<model::CreateTask>,
) -> Result<(StatusCode, Json<u64>), CreateTaskError> {
    task_scheduler
        .add_uri(new_task, true)
        .await
        .map_or(Err(CreateTaskError::Internal), |n| Ok((StatusCode::CREATED, Json(n))))
}

pub async fn list(
    Extension(task_scheduler): Extension<TaskScheduler>,
) -> Result<(StatusCode, Json<Vec<model::TaskStatus>>), GetAllTaskStatusesError> {
    match task_scheduler.get_all_task_statuses().await {
        Ok(events) => Ok((StatusCode::OK, Json(events))),
        Err(source) => {
            tracing::error!("{source}");
            Err(GetAllTaskStatusesError::Internal)
        }
    }
}
