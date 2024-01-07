use axum::http::StatusCode;

pub async fn get_version() -> (StatusCode, String) {
    (StatusCode::OK, caracal_base::PROJECT_SEMVER.to_string())
}
