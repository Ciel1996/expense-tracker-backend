mod currency_api;
mod expense_api;
mod health_api;
mod pot_api;
mod user_api;
mod generate_openapi;

pub mod api {
    use crate::currency_api::currency_api;
    use crate::expense_api::expense_api;
    use crate::health_api::health_api;
    use crate::pot_api::pot_api;
    use crate::user_api::user_api;
    use axum::http::request::Parts;
    use axum::http::StatusCode;
    use axum::Json;
    use expense_tracker_db::setup::DbPool;
    use expense_tracker_services::ExpenseError;
    use utoipa::gen::serde_json::Value;
    use utoipa_axum::router::OpenApiRouter;
    use uuid::Uuid;

    /// The generic response that is returned by APIs.
    pub type ApiResponse<T> = (StatusCode, Json<T>);

    const VERSION_ONE: &str = "/v1";
    const SUB_CLAIM: &str = "sub";
    const PREFERRED_USERNAME_CLAIM: &str = "preferred_username";

    /// Registers the APIs with token validation.
    pub async fn router(pool: DbPool) -> OpenApiRouter {
        OpenApiRouter::new()
            .nest(VERSION_ONE, user_api::register(pool.clone()))
            .nest(VERSION_ONE, pot_api::register(pool.clone()))
            .nest(VERSION_ONE, currency_api::register(pool.clone()))
            .nest(VERSION_ONE, expense_api::register(pool.clone()))
    }

    /// Registers the health API without token validation so it is always possible to
    /// check if the service is running or not.
    pub async fn add_health_api() -> OpenApiRouter {
        OpenApiRouter::new().nest(VERSION_ONE, health_api::register())
    }

    /// Checks the given `Error` and gets the correct error message.
    /// Returns one of:
    /// - 403
    /// - 404
    /// - 409
    /// - 500
    pub fn check_error(err: ExpenseError) -> ApiResponse<String> {
        match err {
            ExpenseError::Forbidden(message) => (StatusCode::FORBIDDEN, Json(message)),
            ExpenseError::NotFound(message) => (StatusCode::NOT_FOUND, Json(message)),
            ExpenseError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, Json(message)),
            ExpenseError::Conflict(message) => (StatusCode::CONFLICT, Json(message)),
        }
    }

    /// Gets the sub_claim from the given Request that has been put there by the auth middleware.
    pub fn get_sub_claim(parts: &Parts) -> Result<Uuid, ApiResponse<String>> {
        let claims: &Value = parts.extensions.get().unwrap();

        let sub_claim = claims
            .get(SUB_CLAIM)
            .expect("Sub claim must be set!")
            .as_str()
            .ok_or(ExpenseError::Internal("Sub claim must be set!".to_string()))
            .map_err(check_error);

        Ok(Uuid::parse_str(sub_claim?).expect("Failed to parse uuid"))
    }

    /// Gets the preferred_username from the given Request that has been put there by the auth middleware.
    pub fn get_username(parts: &Parts) -> Result<String, ApiResponse<String>> {
        let claims: &Value = parts.extensions.get().unwrap();

        let user_name = claims
            .get(PREFERRED_USERNAME_CLAIM)
            .expect("Username must be set!")
            .as_str()
            .ok_or(ExpenseError::Internal("Username must be set!".to_string()))
            .map_err(check_error);

        Ok(user_name?.to_string())
    }

    // pub fn generate_my_openapi() -> String {
    //     #[derive(OpenApi)]
    //     #[openapi(schemas(components(CurrencyDTO)))]
    //     struct ApiDoc;
    //
    //     ApiDoc::openapi().to_pretty_json().unwrap()
    // }

}
