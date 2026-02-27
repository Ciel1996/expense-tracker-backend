pub mod template_api {
    use std::sync::Arc;
    use axum::extract::{Path, State};
    use axum::http::request::Parts;
    use axum::Json;
    use chrono::{DateTime, Utc};
    use hyper::StatusCode;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    use uuid::Uuid;
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_db::template_pots::template_pots::{NewPotTemplate, PotTemplate};
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
            .routes(routes!(delete_pot_template))
            .with_state(shared_state)
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct NewPotTemplateDTO {
        #[schema(max_length = 24)]
        name: String,
        default_currency_id: i32,
        /// A list of user ids that should be automatically added as members of this pot.
        /// The owner does not need to be part of this list, as they are automatically added
        /// by the service.
        user_ids: Vec<Uuid>,
        create_at: DateTime<Utc>
    }

    impl NewPotTemplateDTO {
        pub fn to_db(&self, owner_id: Uuid) -> NewPotTemplate {
            NewPotTemplate::new(
                owner_id,
                self.name.clone(),
                self.default_currency_id,
                self.create_at
            )
        }

        pub fn default_currency_id(&self) -> i32 {
            self.default_currency_id
        }

        pub fn user_ids(&self) -> &Vec<Uuid> {
            &self.user_ids
        }
    }

    #[derive(ToSchema, Serialize)]
    pub struct PotTemplateDTO {
        id: i32,
        owner: UserDTO,
        name: String,
        default_currency: CurrencyDTO,
        create_at: DateTime<Utc>,
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
                // the owner should be in the list of users
                // (as he is automatically added to it, so we get the owner from the list to avoid additional db calls)
                owner: users.iter().find(|u| u.uuid() == pot_template.owner_id()).unwrap().clone(),
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
        State(template_api_state): State<Arc<TemplateApiState>>,
        parts: Parts,
        Json(new_pot_tempalte): Json<NewPotTemplateDTO>
    ) -> Result<ApiResponse<PotTemplateDTO>, ApiResponse<String>> {
        let subject_id = get_sub_claim(&parts)?;

        let result = template_api_state
            .pot_template_service
            .create_template(new_pot_tempalte.to_db(subject_id), new_pot_tempalte.user_ids)
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

    /// Deletes the given template if the user is the owner of the template.
    #[utoipa::path(
        delete,
        path = "/template/{template_id}",
        tag = "Templates",
        responses(
            (status = 204, description = "The pot template has been deleted."),
            (status = 403, description = "Indicates that the user is not authorized to delete the given pot template."),
            (status = 404, description = "Indicates that the desired pot template does not exists."),
            (status = 409, description = "Indicates that the desired pot template can't be deleted.")
        ),
        params(
            ("template_id" = i32, Path, description = "Database id for the pot template.  ")
        ),
        security(
            ("bearer" = [])
        )
    )]
    pub async fn delete_pot_template(
        State(template_api_state): State<Arc<TemplateApiState>>,
        Path(template_id): Path<i32>,
        parts: Parts
    ) -> Result<ApiResponse<String>, ApiResponse<String>> {
        let subject_id = get_sub_claim(&parts)?;

        template_api_state
            .pot_template_service
            .delete_template(template_id, subject_id)
            .await
            .map_err(check_error)?;

        Ok((
            StatusCode::NO_CONTENT,
            Json(format!("Template with id {} has been deleted.", template_id))
        ))
    }
}