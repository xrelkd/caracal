mod static_assets;
mod system;
mod task;

use axum::{routing, Router};

pub fn api_v1_router() -> Router {
    Router::new().nest("/api", Router::new().merge(self::task::v1()).merge(self::system::v1()))
}

pub fn static_assets_router() -> Router {
    Router::new().route("/*path", routing::get(static_assets::static_path))
}
