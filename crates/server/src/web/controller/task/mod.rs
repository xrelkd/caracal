mod v1;

use axum::{routing, Router};

pub fn v1() -> Router {
    Router::new().nest(
        "/v1/task",
        Router::new()
            .route("/", routing::post(self::v1::create))
            .route("/", routing::get(self::v1::list)),
    )
}
