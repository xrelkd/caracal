mod system;
mod task;

use axum::Router;

pub fn api_v1_router() -> Router {
    Router::new().nest("/api", Router::new().merge(task::v1()).merge(system::v1()))
}
