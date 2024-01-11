mod system;
mod task;

use axum::Router;

pub fn api_v1_router() -> Router {
    Router::new().nest("/api", Router::new().merge(self::task::v1()).merge(self::system::v1()))
}
