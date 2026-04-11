#![allow(clippy::needless_for_each)]

use caracal_base::model;
use utoipa::OpenApi;

use crate::web::controller;

#[derive(OpenApi)]
#[openapi(
    paths(
        controller::task::v1::list,
        controller::task::v1::create,
        controller::task::v1::get,
        controller::task::v1::remove,
        controller::task::v1::pause,
        controller::task::v1::pause_all,
        controller::task::v1::resume,
        controller::task::v1::resume_all,
        controller::system::v1::get_version,
    ),
    components(
        schemas(
            model::CreateTask,
            model::ProgressChunk,
            model::TaskState,
            model::TaskStatus,
        )
    ),
    tags(
        (name = "Task", description = "Task management endpoints."),
        (name = "System", description = "System information.")
    ),
)]
pub struct ApiDoc;

#[cfg(test)]
mod tests {

    use utoipa::OpenApi;

    use super::ApiDoc;

    #[test]
    fn generate_openapi_spec_json() {
        let _unused = ApiDoc::openapi()
            .to_pretty_json()
            .expect("Failed to generate OpenAPI spec in JSON format");
    }

    #[test]
    fn generate_openapi_spec_yaml() {
        let _unused =
            ApiDoc::openapi().to_yaml().expect("Failed to generate OpenAPI spec in YAML format");
    }
}
