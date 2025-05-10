pub mod user_api {
    use axum::extract::State;
    use axum::http::{HeaderMap, StatusCode, Uri};
    use axum::Json;
    use axum::response::IntoResponse;
    use axum::routing::get;
    use axum_oidc::{EmptyAdditionalClaims, OidcClaims, OidcRpInitiatedLogout};
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    use uuid::{uuid, Builder, Uuid};
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_db::users::users::User;
    use expense_tracker_services::user_service::user_service;
    use expense_tracker_services::user_service::user_service::UserService;
    use crate::api::{check_error, ApiResponse};

    /// Registers all functions of the Users API.
    pub fn register(pool : DbPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(check_user))
            .routes(routes!(get_users))
            .with_state(user_service::create_service(pool))
    }

    /// The DTO representing a user from DB.
    #[derive(ToSchema, Serialize, Clone, Deserialize)]
    pub struct UserDTO {
        uuid : Uuid,
        name: String
    }

    impl UserDTO {
        pub(crate) fn new(uuid: Uuid, name: String) -> UserDTO {
            Self {
                uuid,
                name
            }
        }
    }

    impl UserDTO {
        pub fn from(user : User) -> UserDTO {
            UserDTO {
                uuid: user.id(),
                name: user.name().to_string()
            }
        }

        pub fn from_vec(users : Vec<User>) -> Vec<UserDTO> {
            let mut dtos = vec!();

            for user in users {
                dtos.push(UserDTO::from(user));
            }

            dtos
        }

        pub fn uuid(&self) -> Uuid {
            self.uuid
        }

        pub fn name(&self) -> &str {
            &self.name
        }
    }

    /// Creates a new user from the given DTO.
    #[utoipa::path(
            get,
            path = "/current_user",
            tag = "Users",
            responses(
                (status = 200, description = "The user does already exist", body = UserDTO),
                (status = 201, description = "The user", body = UserDTO),
                (status = 500, description = "The server error")
            ),
            security(
                ("bearer" = [])
            )
    )]
    pub async fn check_user(
        claims: OidcClaims<EmptyAdditionalClaims>, // TODO: this is the suspect!
        State(service): State<UserService>
    ) -> Result<ApiResponse<UserDTO>, ApiResponse<String>> {
        let user_name =
            claims.preferred_username().expect("Username must be set!");

        let uuid = Uuid::parse_str(claims.subject().as_str())
            .expect("Failed to parse uuid");
        let user = service.get_user_by_id(uuid).await;

        if let Ok(user) = user {
            return Ok((StatusCode::OK, Json(UserDTO::from(user))));
        }

        let new_user = User::new(uuid, user_name.as_str().to_string());

        let res = service
            .create_user(new_user)
            .await
            .map_err(check_error)?;

        Ok((StatusCode::CREATED, Json(UserDTO::from(res))))
    }

    #[utoipa::path(
        get,
        path = "/users",
        tag = "Users",
        responses(
            (status = 200, description = "All users in the database", body = Vec<UserDTO>),
        ),
        security(
                ("bearer" = [])
        )
    )]
    pub async fn get_users(
        State(service) : State<UserService>
    ) -> Result<ApiResponse<Vec<UserDTO>>, ApiResponse<String>> {
        let res = service.get_users().await.map_err(check_error)?;
        Ok((StatusCode::OK, Json(UserDTO::from_vec(res))))
    }

}