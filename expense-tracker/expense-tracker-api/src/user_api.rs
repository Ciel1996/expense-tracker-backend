pub mod user_api {
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::Json;
    use diesel::{RunQueryDsl, SelectableHelper};
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    use expense_tracker_db::setup::{DbConnectionPool, DbPool};
    use expense_tracker_db::schema as expense_tracker_db_schema;
    use expense_tracker_db::users::users::{NewUser, User};
    use crate::api::internal_error;

    /// Registers all functions of the Users API.
    pub fn register(pool : DbConnectionPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(create_user))
            .with_state(pool)
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
    #[derive(ToSchema, Serialize)]
    pub struct UserDTO {
        id : i32,
        name: String
    }

    impl UserDTO {
        pub fn from(user : User) -> UserDTO {
            UserDTO {
                id: user.id(),
                name: user.name().to_string()
            }
        }
    }

    /// Creates a new user from the given DTO.
    #[utoipa::path(
            post,
            path = "/users",
            tag = "Users",
            responses(
                (status = 200, description = "The user")
            ),
            request_body = NewUserDTO
    )]
    pub async fn create_user(
        State(pool): State<DbPool>,
        Json(new_user): Json<NewUserDTO>
    ) -> Result<Json<UserDTO>, (StatusCode, String)> {
        let conn = pool.get().await.map_err(internal_error)?;

        let res = conn
            .interact(move |conn| {
                diesel::insert_into(expense_tracker_db_schema::users::table)
                    .values(new_user.to_db())
                    .returning(User::as_returning())
                    .get_result::<User>(conn)
            })
            .await
            .map_err(internal_error)?
            .map_err(internal_error)?;

        Ok(Json(UserDTO::from(res)))
    }
}