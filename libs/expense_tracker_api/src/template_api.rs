pub mod template_api {
    use std::sync::Arc;
    use axum::extract::State;
    use axum::http::request::Parts;
    use axum::Json;
    use hyper::StatusCode;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    use uuid::Uuid;
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_db::template_pots::template_pots::{NewPotTemplate, PotTemplate, PotTemplateUser};
    use expense_tracker_services::template_service::pot_template_service::PotTemplateService;
    use crate::api::{check_error, get_sub_claim, ApiResponse};
    use crate::currency_api::currency_api::CurrencyDTO;
    use crate::user_api::user_api::UserDTO;

    pub struct TemplateApiState {
        pot_template_service: PotTemplateService
    }

    /// Registers all functions of the Template API.
    pub fn register(pool: DbPool) -> OpenApiRouter {
        let shared_state = Arc::new(TemplateApiState {
            pot_template_service: PotTemplateService::new_service(pool)
        });

        OpenApiRouter::new()
            .routes(routes!(create_pot_template))
            .with_state(shared_state)
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct NewPotTemplateDTO {
        #[schema(max_length=24)]
        name: String,
        default_currency_id: i32
    }

    impl NewPotTemplateDTO {
        pub fn to_db(&self, owner_id: Uuid) -> NewPotTemplate {
            NewPotTemplate::new(owner_id, self.name.clone(), self.default_currency_id)
        }
    }

    #[derive(ToSchema, Serialize)]
    pub struct PotTemplateDTO {
        id: i32,
        owner_id: Uuid,
        name: String,
        default_currency: CurrencyDTO,
        create_at: Option<chrono::DateTime<chrono::Utc>>,
        users: Vec<UserDTO>
    }

    impl PotTemplateDTO {
        pub fn from(
            pot_template: PotTemplate,
            default_currency: CurrencyDTO,
            users: Vec<UserDTO>) -> Self
        {
            Self {
                id: pot_template.id(),
                owner_id: pot_template.owner_id(),
                name: pot_template.name().to_string(),
                default_currency,
                create_at: pot_template.create_at(),
                users
            }
        }
    }

    /// Creates a pot template from the given DTO for the bearer.
    #[utoipa::path(
        post,
        path = "/template",
        tag = "Templates",
        responses(
            (status = 201, description = "The pot template has been created", body = PotTemplateDTO),
        ),
        request_body = NewPotTemplateDTO,
        security(
            ("bearer" = [])
        )
    )]
    pub async fn create_pot_template(
        State(template_api_state) : State<Arc<TemplateApiState>>,
        parts: Parts,
        Json(new_pot_tempalte): Json<NewPotTemplateDTO>
    ) -> Result<ApiResponse<PotTemplateDTO>, ApiResponse<String>> {
        let subject_id = get_sub_claim(&parts)?;

        let result = template_api_state
            .pot_template_service
            .create_template(new_pot_tempalte.to_db(subject_id), vec![])
            .await
            .map_err(check_error)?;

        Ok((
            StatusCode::CREATED,
            Json(PotTemplateDTO::from(
                result.0,
                CurrencyDTO::from(result.1),
                UserDTO::from_vec(result.2)
            ))
        ))
    }
}