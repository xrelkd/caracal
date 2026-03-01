pub mod api_doc;

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use self::api_doc::ApiDoc;

pub fn ui_router() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi())
}
