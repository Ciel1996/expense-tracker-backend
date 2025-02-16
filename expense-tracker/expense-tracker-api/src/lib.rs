mod health_api;
mod user_api;
mod pot_api;
mod currency_api;

pub mod api {
    use std::error::Error;
    use std::ops::Deref;
    use axum::http::StatusCode;
    use utoipa_axum::router::OpenApiRouter;
    use expense_tracker_db::setup::DbConnectionPool;
    use expense_tracker_services::NotFoundError;
    use crate::currency_api::currency_api;
    use crate::health_api::health_api;
    use crate::pot_api::pot_api;
    use crate::user_api::user_api;

    const VERSION_ONE : &str = "/v1";

    pub fn router(pool: DbConnectionPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .nest(VERSION_ONE, health_api::register())
            .nest(VERSION_ONE, user_api::register(pool.clone()))
            .nest(VERSION_ONE, pot_api::register(pool.clone()))
            .nest(VERSION_ONE, currency_api::register(pool.clone()))
    }

    /// Utility function for mapping any error into a `500 Internal Server Error`
    /// response.
    pub fn internal_error<E>(err: E) -> (StatusCode, String)
    where
        E: std::error::Error,
    {
        (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }

    // TODO: get rid of old function and rename the following!
    /// Utility function for mapping any error into a '500 Internal Server Error' response.
    pub fn internal_error_new(err: String) -> (StatusCode, String) {
        (StatusCode::INTERNAL_SERVER_ERROR, err)
    }

    pub fn check_error(err : Box<dyn Error>) -> (StatusCode, String) {
        if err.is::<NotFoundError>() {
            (StatusCode::NOT_FOUND, err.to_string())
        } else {
            internal_error(Err(err))
        }
    }
}