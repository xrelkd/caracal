use axum::http::StatusCode;

#[utoipa::path(
    get,
    path = "/api/v1/system/version",
    responses(
      (
        status = 200,
        description = "Retrieve the version of Caracal",
        body = String,
        example = "0.3.0"
      ),
    ),
    tag = "System"
)]
pub async fn get_version() -> (StatusCode, String) {
    (StatusCode::OK, caracal_base::PROJECT_SEMVER.to_string())
}
