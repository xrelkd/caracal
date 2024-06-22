mod v1;

use axum::{routing, Router};

pub fn v1() -> Router {
    Router::new().nest("/v1/system", Router::new().route("/version", routing::get(v1::get_version)))
}
