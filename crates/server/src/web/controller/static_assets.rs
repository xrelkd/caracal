use axum::{
    body,
    extract::Path,
    response::{IntoResponse, Response},
};
use http::{header, HeaderValue, StatusCode};

pub async fn static_path(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    let mime_type = mime_guess::from_path(path).first_or_text_plain();
    crate::FRONTEND_STATIC_ASSETS_DIR.get_file(path).map_or_else(
        || {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(body::Body::from(()))
                .expect("building response with empty body should always success")
        },
        |file| {
            let content_type =
                HeaderValue::from_str(mime_type.as_ref()).expect("`mime_type` is valid string");
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, content_type)
                .body(body::Body::from(file.contents()))
                .expect("building response with such contents should always success")
        },
    )
}
