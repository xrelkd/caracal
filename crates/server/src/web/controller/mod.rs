pub mod system;
pub mod task;

use axum::{
    Router, body, http,
    response::{IntoResponse, Response},
};

const INDEX_PAGE_ASSET_NAME: &str = "index.html";
const NOT_FOUND_PAGE_ASSET_NAME: &str = "404.html";

pub fn api_v1_router() -> Router {
    Router::new().nest("/api", Router::new().merge(task::v1()).merge(system::v1()))
}

pub async fn web_ui(uri: http::Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { INDEX_PAGE_ASSET_NAME } else { path };

    caracal_web_ui::WEB_UI_ASSETS.get(path).map_or_else(
        || {
            caracal_web_ui::WEB_UI_ASSETS.get(NOT_FOUND_PAGE_ASSET_NAME).map_or_else(
                || http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
                |content| {
                    Response::builder()
                        .status(http::StatusCode::NOT_FOUND)
                        .header(http::header::CONTENT_TYPE, content.mime_type)
                        .body(body::Body::from(content.data))
                        .expect("response is valid")
                },
            )
        },
        |content| {
            Response::builder()
                .header(http::header::CONTENT_TYPE, content.mime_type)
                .body(body::Body::from(content.data))
                .expect("response is valid")
        },
    )
}
