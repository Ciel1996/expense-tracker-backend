pub mod api {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;

    pub fn router() -> OpenApiRouter {
        OpenApiRouter::new()
            .routes(routes!(health_check))
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

    pub async fn health_check() -> impl IntoResponse {
        (StatusCode::OK, "Ok").into_response()
    }
}