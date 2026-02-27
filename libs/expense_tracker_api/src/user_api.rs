pub mod user_api {
    use crate::api::{check_error, get_sub_claim, get_username, ApiResponse};
    use axum::body::Body;
    use axum::extract::State;
    use axum::http::{Request, StatusCode};
    use axum::Json;
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_db::users::users::User;
    use expense_tracker_services::user_service::user_service;
    use expense_tracker_services::user_service::user_service::UserService;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    use uuid::Uuid;

    /// Registers all functions of the Users API.
    pub fn register(pool: DbPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(current_user))
            .routes(routes!(get_users))
            .with_state(user_service::new_service(pool))
    }

    /// The DTO representing a user from DB.
    #[derive(ToSchema, Serialize, Clone, Deserialize)]
    pub struct UserDTO {
        uuid: Uuid,
        name: String,
    }

    impl UserDTO {
        pub fn new(uuid: Uuid, name: String) -> Self {
            Self { uuid, name }
        }

        pub fn from(user: User) -> UserDTO {
            Self::new(user.id(),user.name().to_string())
        }

        pub fn from_vec(users: Vec<User>) -> Vec<UserDTO> {
            let mut dtos = vec![];

            for user in users {
                dtos.push(UserDTO::from(user));
            }

            dtos
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
    pub async fn current_user(
        State(service): State<UserService>,
        request: Request<Body>,
    ) -> Result<ApiResponse<UserDTO>, ApiResponse<String>> {
        let (parts, _) = request.into_parts();
        let uuid = get_sub_claim(&parts)?;
        let user = service.get_user_by_id(uuid).await;

        // TODO: what happens in case of a DB exception?
        if let Ok(user) = user {
            return Ok((StatusCode::OK, Json(UserDTO::from(user))));
        }

        let user_name = get_username(&parts)?;
        let new_user = User::new(uuid, user_name);

        let res = service.create_user(new_user).await.map_err(check_error)?;

        Ok((StatusCode::CREATED, Json(UserDTO::from(res))))
    }

    /// Returns a list of a users registered in the system. Needs a bearer token to track
    /// who requested it.
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
        State(service): State<UserService>,
    ) -> Result<ApiResponse<Vec<UserDTO>>, ApiResponse<String>> {
        let res = service.get_users().await.map_err(check_error)?;
        Ok((StatusCode::OK, Json(UserDTO::from_vec(res))))
    }
}
