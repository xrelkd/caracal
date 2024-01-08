use axum::{
    body,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Clone, Debug)]
pub enum CreateTaskError {
    Internal,
}

impl IntoResponse for CreateTaskError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, body::Body::from(())),
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
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, body::Body::from(())),
        };

        Response::builder()
            .status(status)
            .body(body)
            .expect("response should always build successfully")
    }
}

#[derive(Clone, Debug)]
pub enum GetTaskError {
    NotFound,
    Internal,
}

impl IntoResponse for GetTaskError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, body::Body::from(())),
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, body::Body::from(())),
        };

        Response::builder()
            .status(status)
            .body(body)
            .expect("response should always build successfully")
    }
}

#[derive(Clone, Debug)]
pub enum RemoveTaskError {
    NotFound,
    Internal,
}

impl IntoResponse for RemoveTaskError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, body::Body::from(())),
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, body::Body::from(())),
        };

        Response::builder()
            .status(status)
            .body(body)
            .expect("response should always build successfully")
    }
}

#[derive(Clone, Debug)]
pub enum ResumeTaskStatusesError {
    NotFound,
    Internal,
}

impl IntoResponse for ResumeTaskStatusesError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, body::Body::from(())),
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, body::Body::from(())),
        };

        Response::builder()
            .status(status)
            .body(body)
            .expect("response should always build successfully")
    }
}

#[derive(Clone, Debug)]
pub enum ResumeAllTasksError {
    Internal,
}

impl IntoResponse for ResumeAllTasksError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, body::Body::from(())),
        };

        Response::builder()
            .status(status)
            .body(body)
            .expect("response should always build successfully")
    }
}

#[derive(Clone, Debug)]
pub enum PauseTaskError {
    NotFound,
    Internal,
}

impl IntoResponse for PauseTaskError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, body::Body::from(())),
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, body::Body::from(())),
        };

        Response::builder()
            .status(status)
            .body(body)
            .expect("response should always build successfully")
    }
}

#[derive(Clone, Debug)]
pub enum PauseAllTasksError {
    Internal,
}

impl IntoResponse for PauseAllTasksError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            Self::Internal => (StatusCode::INTERNAL_SERVER_ERROR, body::Body::from(())),
        };

        Response::builder()
            .status(status)
            .body(body)
            .expect("response should always build successfully")
    }
}
