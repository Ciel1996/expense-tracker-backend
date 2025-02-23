mod health_api;
mod user_api;
mod pot_api;
mod currency_api;

pub mod api {
    use axum::http::StatusCode;
    use axum::Json;
    use utoipa_axum::router::OpenApiRouter;
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_services::ExpenseError;
    use crate::currency_api::currency_api;
    use crate::health_api::health_api;
    use crate::pot_api::pot_api;
    use crate::user_api::user_api;

    /// The generic response that is returned by APIs.
    pub type ApiResponse<T> = (StatusCode, Json<T>);

    const VERSION_ONE: &str = "/v1";

    pub fn router(pool: DbPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .nest(VERSION_ONE, health_api::register())
            .nest(VERSION_ONE, user_api::register(pool.clone()))
            .nest(VERSION_ONE, pot_api::register(pool.clone()))
            .nest(VERSION_ONE, currency_api::register(pool.clone()))
    }

    /// Checks the given `Error` and gets the correct error message.
    /// Returns one of:
    /// - 404
    /// - 409
    /// - 500
    pub fn check_error(err: ExpenseError) -> ApiResponse<String>
    {
        match err {
            ExpenseError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                Json(message)
            ),
            ExpenseError::Internal(message) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(message)
            ),
            ExpenseError::Conflict(message) => (
                StatusCode::CONFLICT,
                Json(message)
            ),
        }
    }
}