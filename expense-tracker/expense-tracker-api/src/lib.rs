mod health;
mod users;

pub mod api {
    use expense_tracker_db::setup::DbConnectionPool;
    use axum::http::StatusCode;
    use utoipa_axum::router::OpenApiRouter;
    use crate::health::health_api;
    use crate::users::user_api;

    pub fn router(pool: DbConnectionPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .nest("", health_api::register(pool.clone()))
            .nest("", user_api::register(pool.clone()))
    }

    /// Utility function for mapping any error into a `500 Internal Server Error`
    /// response.
    pub fn internal_error<E>(err: E) -> (StatusCode, String)
    where
        E: std::error::Error,
    {
        (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}