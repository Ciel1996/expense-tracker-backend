pub mod template_api {
    use std::collections::HashSet;
    use std::sync::Arc;
    use axum::extract::{Path, State};
    use axum::http::request::Parts;
    use axum::Json;
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
    pub async fn register(pool: DbPool) -> OpenApiRouter {
        let shared_state = Arc::new(TemplateApiState {
            pot_template_service: PotTemplateService::new_service(pool)
        });

        shared_state.pot_template_service.init_service().await;

        OpenApiRouter::new()
            .routes(routes!(create_pot_template))
            .routes(routes!(delete_pot_template))
            .routes(routes!(add_users_to_template))
            .routes(routes!(remove_users_from_template))
            .with_state(shared_state)
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct NewPotTemplateDTO {
        #[schema(max_length = 24)]
        #[schema(example = "My Pot {month}.{year}")]
        /// Name for the pot template. Supports placeholders for the current month {month} and year {year}.
        name: String,
        default_currency_id: i32,
        /// A list of user ids that should be automatically added as members of this pot.
        /// The owner does not need to be part of this list, as they are automatically added
        /// by the service.
        user_ids: Vec<Uuid>,
        /// A cron expression that defines when the pot should be automatically created.
        /// The example shows how a cron expression must look that expresses "At 12:00 AM, on day 1 of the month"
        /// I recommend using tools like https://crontab.cronhub.io to generate and validate a cron expression.
        /// Expense-Tracker uses the local timezone to make it easier for the user to think in their local timezone.
        #[schema(example = "0 0 0 1 * *")]
        cron_expression: String
    }

    impl NewPotTemplateDTO {
        pub fn to_db(&self, owner_id: Uuid) -> NewPotTemplate {
            NewPotTemplate::new(
                owner_id,
                self.name.clone(),
                self.default_currency_id,
                self.cron_expression.clone()
            )
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
        /// The example shows how a cron expression must look that expresses "At 12:00 AM, on day 1 of the month"
        /// I recommend using tools like https://crontab.cronhub.io to generate and validate a cron expression.
        /// Expense-Tracker uses the local timezone to make it easier for the user to think in their local timezone.
        #[schema(example = "0 0 0 1 * *")]
        cron_expression: String,
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
                cron_expression: pot_template.cron_expression().to_string(),
                users
            }
        }
    }

    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct UserListDTO {
        users: Vec<Uuid>,
    }

    impl UserListDTO {
        pub fn users(&self) -> Vec<Uuid> {
            // deduplicating list on get
            let mut users_hash_set = HashSet::new();

            for u in self.users.iter().cloned() {
                users_hash_set.insert(u);
            }

            users_hash_set.into_iter().collect::<Vec<Uuid>>()
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
            .create_template(new_pot_tempalte.to_db(subject_id), new_pot_tempalte.user_ids().clone())
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

    /// Adds the given users to the given template if the calling user is the owner of the template.
    #[utoipa::path(
        put,
        path = "/template/{template_id}/users/add",
        tag = "Templates",
        responses(
            (status = 202, description = "The users has been added to the pot template."),
            (status = 403, description = "Indicates that the users is not authorized to add users to the given pot template."),
            (status = 404, description = "Indicates that the desired users or pot template does not exists."),
            (status = 409, description = "Indicates that the desired users can't be added to the pot template.")
        ),
        request_body = UserListDTO,
        params(
            ("template_id" = i32, Path, description = "Database id for the pot template.  ")
        ),
        security(
            ("bearer" = [])
        )
    )]
    pub async fn add_users_to_template(
        State(template_api_state): State<Arc<TemplateApiState>>,
        Path(template_id): Path<i32>,
        parts: Parts,
        Json(add_users_dto): Json<UserListDTO>
    ) -> Result<ApiResponse<String>, ApiResponse<String>> {
        let subject_id = get_sub_claim(&parts)?;

        template_api_state
            .pot_template_service
            .add_users_to(template_id, add_users_dto.users(), subject_id)
            .await
            .map_err(check_error)?;

        Ok((
            StatusCode::ACCEPTED,
            Json(format!("The users have been added to the template with id {}.", template_id))
        ))
    }

    /// Removes the given users from the given template if the calling user is the owner of the template.
    #[utoipa::path(
        put,
        path = "/template/{template_id}/users/remove",
        tag = "Templates",
        responses(
            (status = 202, description = "The users have been removed from the pot template."),
            (status = 403, description = "Indicates that the user is not authorized to remove users from the given pot template."),
            (status = 404, description = "Indicates that the desired users or pot template does not exists."),
            (status = 409, description = "Indicates that the desired users can't be removed from the pot template.")
        ),
        request_body = UserListDTO,
        params(
            ("template_id" = i32, Path, description = "Database id for the pot template.  ")
        ),

        security(
            ("bearer" = [])
        )
    )]
    pub async fn remove_users_from_template(
        State(template_api_state): State<Arc<TemplateApiState>>,
        Path(template_id): Path<i32>,
        parts: Parts,
        Json(remove_users_dto): Json<UserListDTO>
    ) -> Result<ApiResponse<String>, ApiResponse<String>> {
        let subject_id = get_sub_claim(&parts)?;

        template_api_state
            .pot_template_service
            .remove_users_from(template_id, remove_users_dto.users(), subject_id)
            .await
            .map_err(check_error)?;

        Ok((
            StatusCode::ACCEPTED,
            Json(format!("The users have been removed from the template with id {}.", template_id))
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