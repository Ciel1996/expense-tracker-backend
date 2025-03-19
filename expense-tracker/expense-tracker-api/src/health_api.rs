pub mod health_api {
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    use expense_tracker_services::health_service::health_service;
    use expense_tracker_services::health_service::health_service::PingHealthService;

    /// Registers all functions of the health API.
    pub fn register() -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(health_check))
            .with_state(health_service::new_service())
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
    pub async fn health_check(State(service) : State<PingHealthService>) -> impl IntoResponse {
        (StatusCode::OK, service.ping()).into_response()
    }
}