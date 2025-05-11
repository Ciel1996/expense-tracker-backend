extern crate core;

use std::net::SocketAddr;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use cookie::SameSite;
use jsonwebtoken::{decode, DecodingKey, Validation};
use jsonwebtoken::jwk::JwkSet;
use reqwest::get;
use time::Duration;
use tower::ServiceBuilder;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use utoipa::{Modify, OpenApi};
use utoipa::gen::serde_json::Value;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;
use utoipa_swagger_ui::oauth;
use expense_tracker_api::api;
use expense_tracker_db::setup::setup_db;
use log::{debug, error, info};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)))
        }
    }
}

/// Fetches the jwks used for validation of the token
async fn fetch_jwks(jwks_url: &str) -> Result<JwkSet, reqwest::Error> {
    let response = get(jwks_url).await?;
    let jwks = response.json::<JwkSet>().await?;
    Ok(jwks)
}

async fn validate_token(token: &str) -> Result<Value, String> {
    debug!("Starting token validation");
    // TODO: load jwks url from config (issuer url + /protocol/openid-connect/certs)
    let jwks_url = "http://localhost:8080/realms/expense-tracker-dev/protocol/openid-connect/certs";
    let jwks = match fetch_jwks(jwks_url).await {
        Ok(jwks) => jwks,
        Err(e) => {
            error!("Failed to fetch JWKS: {}", e);
            return Err("Failed to fetch JWKS".to_string());
        }
    };

    let header = match jsonwebtoken::decode_header(token) {
        Ok(header) => header,
        Err(e) => {
            error!("Failed to decode token header: {}", e);
            return Err("Failed to decode token header".to_string());
        }
    };

    let kid = match header.kid {
        Some(kid) => kid,
        None => {
            error!("No kid in header");
            return Err("No kid in header".to_string());
        }
    };

    let key = match jwks.find(&kid) {
        Some(key) => key,
        None => {
            error!("Key not found in JWKS");
            return Err("Key not found in JWKS".to_string());
        }
    };

    let decoding_key = match DecodingKey::from_jwk(key) {
        Ok(key) => key,
        Err(e) => {
            error!("Failed to create decoding key: {}", e);
            return Err("Failed to create decoding key".to_string());
        }
    };

    let mut validation = Validation::new(header.alg);
    // TODO: read from config
    // TODO: validate as much as possible
    validation.set_audience(&["expense-tracker"]);
    validation.set_issuer(&["http://localhost:8080/realms/expense-tracker-dev"]);
    // validation.set_required_spec_claims()
    match decode::<Value>(token, &decoding_key, &validation) {
        Ok(data) => {
            debug!("Token validated successfully");
            Ok(data.claims)
        }
        Err(e) => {
            error!("Failed to decode token: {}", e);
            Err(format!("Failed to decode token: {e}"))
        }
    }
}

async fn auth_middleware(request: Request<Body>, next: Next) -> Result<Response, Response<String>> {
    debug!("Auth middleware entered!");
    let (parts, body) = request.into_parts();

    let token = parts
        .headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(||
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body("Unauthorized, no Bearer token present".to_string()).unwrap())?;

    debug!("Token extracted!");

    let claims = validate_token(token)
        .await
        .map_err(|e|
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(format!("Unauthorized, {e}")).unwrap())?;

    // Insert claims into request extensions for use in handlers
    let mut parts = parts;
    parts.extensions.insert(claims);

    let request = Request::from_parts(parts, body);
    Ok(next.run(request).await)
}

#[tokio::main]
async fn main() {
    // 1. Initialize tracing + log bridging
    tracing_subscriber::fmt()
        // This allows you to use, e.g., `RUST_LOG=info` or `RUST_LOG=debug`
        // when running the app to set log levels.
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("expense-tracker=error,tower_http=warn"))
                .unwrap(),
        )
        .init();

    let pool = setup_db().await.expect("Failed to create pool");

    let session_store = MemoryStore::default();
    let session_layer =
        SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_same_site(SameSite::Lax)
            .with_expiry(Expiry::OnInactivity(Duration::seconds(60)));

    // To get a JWT: curl -X POST 'http://localhost:8080/realms/expense-tracker-dev/protocol/openid-connect/token' -H 'Content-Type: application/x-www-form-urlencoded' -d 'client_id=<CLIENT_ID>' -d 'username=<USER>' -d 'password=<PASSWORD>' -d 'grant_type=password' -d 'scope=email profile' -d 'client_secret=<CLIENT_SECRET>'

    let oauth_validator = ServiceBuilder::new()
        .layer(axum::middleware::from_fn(auth_middleware));

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", api::router(pool).await)
        .layer(oauth_validator)
        .layer(session_layer)
        .nest("/api", api::add_health_api().await)
        // 3. Add a TraceLayer to automatically create and enter spans
        .layer(TraceLayer::new_for_http())
        .split_for_parts();

    // setup oAuth with utoipa swagger ui
    let oauth_config = oauth::Config::new()
        .client_id("expense-tracker");

    let router = router
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", api.clone())
            .oauth(oauth_config));

    // TODO: read from config
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router.into_make_service()).await.unwrap();
}
