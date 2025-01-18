mod health_api;
mod user_api;
mod pot_api;
mod currency_api;

pub mod api {
    use axum::http::StatusCode;
    use utoipa_axum::router::OpenApiRouter;
    use expense_tracker_db::setup::DbConnectionPool;
    use crate::currency_api::currency_api;
    use crate::health_api::health_api;
    use crate::pot_api::pot_api;
    use crate::user_api::user_api;

    pub fn router(pool: DbConnectionPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .nest("", health_api::register(pool.clone()))
            .nest("", user_api::register(pool.clone()))
            .nest("", pot_api::register(pool.clone()))
            .nest("", currency_api::register(pool.clone()))
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