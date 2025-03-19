use std::net::SocketAddr;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;
use expense_tracker_api::api;
use expense_tracker_db::setup::setup_db;

#[derive(OpenApi)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let pool = setup_db().await.expect("Failed to create pool");

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", api::router(pool))
        .split_for_parts();

    let router = router
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", api.clone()));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router.into_make_service()).await.unwrap();
}
