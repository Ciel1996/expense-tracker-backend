pub mod template_api {
    use serde::{Deserialize, Serialize};
    use utoipa_axum::router::OpenApiRouter;
    use uuid::Uuid;
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_services::template_service::pot_template_service::PotTemplateService;

    pub struct TemplateApiState {
        pot_template_service: PotTemplateService
    }

    /// Registers all functions of the Template API.
    pub fn register(pool: DbPool) -> OpenApiRouter {
        OpenApiRouter::new()
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct NewPotTemplateDTO {
        owner_id: Uuid,
        #[schema(max_length=24)]
        name: String
    }

    pub async fn create_template() {

    }
}