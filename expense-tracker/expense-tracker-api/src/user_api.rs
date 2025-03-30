pub mod user_api {
    use axum::extract::State;
    use axum::http::{StatusCode, Uri};
    use axum::Json;
    use axum::response::IntoResponse;
    use axum::routing::get;
    use axum_oidc::{EmptyAdditionalClaims, OidcClaims, OidcRpInitiatedLogout};
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_db::users::users::{NewUser, User};
    use expense_tracker_services::user_service::user_service;
    use expense_tracker_services::user_service::user_service::UserService;
    use crate::api::{check_error, ApiResponse};

    /// Registers all functions of the Users API.
    pub fn register(pool : DbPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(create_user))
            .routes(routes!(get_users))
            .route("/login", get(login))
            .route("/logout", get(logout))
            .with_state(user_service::create_service(pool))
    }

    /// The DTO representing a new user to be created.
    #[derive(ToSchema, Serialize, Deserialize)]
    pub struct NewUserDTO {
        name: String
    }

    impl NewUserDTO {
        fn to_db(&self) -> NewUser {
            NewUser::new(self.name.clone())
        }
    }

    /// The DTO representing a user from DB.
    #[derive(ToSchema, Serialize, Clone, Deserialize)]
    pub struct UserDTO {
        id : i32,
        name: String
    }

    impl UserDTO {
        pub(crate) fn new(id: i32, name: String) -> UserDTO {
            Self {
                id,
                name
            }
        }
    }

    impl UserDTO {
        pub fn from(user : User) -> UserDTO {
            UserDTO {
                id: user.id(),
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

        pub fn id(&self) -> i32 {
            self.id
        }

        pub fn name(&self) -> &str {
            &self.name
        }
    }

    /// Creates a new user from the given DTO.
    #[utoipa::path(
            post,
            path = "/users",
            tag = "Users",
            responses(
                (status = 201, description = "The user", body = UserDTO),
                (status = 500, description = "The server error")
            ),
            request_body = NewUserDTO
    )]
    pub async fn create_user(
        State(service): State<UserService>,
        Json(new_user): Json<NewUserDTO>
    ) -> Result<ApiResponse<UserDTO>, ApiResponse<String>> {
        let res = service
            .create_user(new_user.to_db())
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
        )
    )]
    pub async fn get_users(
        State(service) : State<UserService>
    ) -> Result<ApiResponse<Vec<UserDTO>>, ApiResponse<String>> {
        let res = service.get_users().await.map_err(check_error)?;
        Ok((StatusCode::OK, Json(UserDTO::from_vec(res))))
    }

    #[utoipa::path(
        get,
        path = "/login",
        tag = "Users",
        responses()
    )]
    pub async fn login(claims: OidcClaims<EmptyAdditionalClaims>) -> impl IntoResponse {
        let first_name = claims.given_name();
        let last_name = claims.family_name();

        let user_name = claims
            .preferred_username().expect("Username").to_string();

        let first_name = match first_name {
            Some(first_name) => first_name
                .get(None).expect("first name").to_string(),
            None => "n/a".to_string()
        };

        let last_name = match last_name {
            Some(last_name) => last_name
                .get(None).expect("last name").to_string(),
            None => "n/a".to_string()
        };

        let email = match claims.email() {
            Some(email) => email.to_string(),
            None => "n/a".to_string()
        };

        // TODO: use this to check if user already exists in DB
        // TODO: if not, create a user entry, otherwise it's fine
        let user_id = claims.subject().as_str();

        let is_verified = claims.email_verified().unwrap_or_else(|| { false });

        format!(
            "Hello {} {} ({}) ({})\nYour id is {}\nYou are {}",
            first_name,
            last_name,
            user_name,
            email,
            user_id,
            if is_verified { "verified".to_string() } else { "unverified".to_string() }
        ).into_response()
    }

    pub async fn logout(logout: OidcRpInitiatedLogout) -> impl IntoResponse {
        logout.with_post_logout_redirect(Uri::from_static("https://cielcat.ch"))
    }
}