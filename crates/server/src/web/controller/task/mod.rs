mod v1;

use axum::{routing, Router};

pub fn v1() -> Router {
    Router::new().nest(
        "/v1/task",
        Router::new()
            .route("/", routing::post(v1::create))
            .route("/", routing::get(v1::list))
            .route("/pause/:task_id", routing::post(v1::pause))
            .route("/pause/", routing::post(v1::pause_all))
            .route("/resume/:task_id", routing::post(v1::resume))
            .route("/resume/", routing::post(v1::resume_all))
            .route("/remove/:task_id", routing::delete(v1::remove))
            .route("/:task_id", routing::get(v1::get)),
    )
}
