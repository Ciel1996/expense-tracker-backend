use std::net::SocketAddr;
use axum::error_handling::HandleErrorLayer;
use axum::http::Uri;
use axum::response::IntoResponse;
use axum_oidc::error::MiddlewareError;
use axum_oidc::{EmptyAdditionalClaims, OidcAuthLayer, OidcLoginLayer};
use cookie::SameSite;
use time::Duration;
use tower::ServiceBuilder;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
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

    let session_store = MemoryStore::default();
    let session_layer =
        SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_same_site(SameSite::Lax)
            .with_expiry(Expiry::OnInactivity(Duration::seconds(60)));

    let oidc_login_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: MiddlewareError| async {
            e.into_response()
        }))
        .layer(OidcLoginLayer::<EmptyAdditionalClaims>::new());

    // TODO: read from config
    let uri = Uri::from_maybe_shared("http://localhost:3000/")
        .expect("valid APP_URL");
    let issuer = "http://localhost:8080/realms/expense-tracker-dev".to_string();
    let client_id = "expense-tracker".to_string();
    let client_secret = Some("ebgzm9OxhLLtyamkGfC3BQpZZmxHvlcU".to_string());
    let scopes = vec![];

    println!("APP_URL = {uri}");

    let oidc_auth_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: MiddlewareError| async {
            e.into_response()
        }))
        .layer(OidcAuthLayer::<EmptyAdditionalClaims>::discover_client(
            uri,
            issuer,
            client_id,
            client_secret,
            scopes
        )
            .await
            .unwrap()
        );

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", api::router(pool).await)
        .layer(oidc_login_service)
        .layer(oidc_auth_service)
        .layer(session_layer)
        .split_for_parts();

    let router = router
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", api.clone()));

    // TODO: read from config
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router.into_make_service()).await.unwrap();
}
