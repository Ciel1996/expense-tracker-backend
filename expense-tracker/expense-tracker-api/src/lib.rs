pub mod api {
    use axum::body::Body;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    use expense_tracker_db::setup::DbConnectionPool;
    use axum::extract::{Query,State};
    use axum::handler::Handler;
    use axum::http::StatusCode;
    use axum::Json;
    use axum::response::IntoResponse;
    use diesel::associations::HasTable;
    use diesel::{RunQueryDsl, SelectableHelper};
    use serde::Serialize;
    use utoipa::ToSchema;
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_db::users::{NewUser, User};
    use expense_tracker_db::users::users::dsl::users;

    pub fn router(pool: DbConnectionPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(
                health_check
            ))
            .routes(routes!(get_user))
            .routes(routes!(create_user))
            .with_state(pool)
    }

    /// HealthCheck Url
    #[utoipa::path(
        get,
        path = "/health",
        tag = "Health",
        responses(
            (status = 200, description = "If this can be reached, the API is available.")
        )
    )]
    async fn health_check() -> impl IntoResponse {
        (StatusCode::OK, "Ok").into_response()
    }

    #[utoipa::path(
        get,
        path = "/users",
        tag = "Users",
        responses(
            (status = 200, description = "The user")
        ),
        params(
            ("test" = i32, Query)
        )
    )]
    // TODO: this seems to be working a lot better
    async fn get_user(Query(test) : Query<i32>) -> impl IntoResponse {
        (StatusCode::OK, Json(UserDTO {
            id: 1,
            name: String::from("Dennis")
        })).into_response()
    }

    /// The DTO representing a new user to be created.
    #[derive(ToSchema, Serialize)]
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
            (status = ACCEPTED, description = "User has been created"),
        ),
        request_body = NewUserDTO
    )]
    async fn create_user(
        State(pool): State<DbPool>,
        Json(new_user): Json<NewUserDTO>,
    ) -> impl IntoResponse {
        let conn = pool.get().await.unwrap();
        let res = conn
            .interact(move |conn| {
                diesel::insert_into(users::table())
                    .values(new_user.to_db())
                    .returning(User::as_returning())
                    .get_result::<User>(conn)
            })
            .await
            .unwrap()
            .unwrap();

        (StatusCode::ACCEPTED, Json(UserDTO::from(res))).into_response()
    }

    /// Utility function for mapping any error into a `500 Internal Server Error`
    /// response.
    fn internal_error<E>(err: E) -> (StatusCode, String)
    where
        E: std::error::Error,
    {
        (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}