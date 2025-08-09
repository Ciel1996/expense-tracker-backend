extern crate core;
mod settings;

use crate::settings::Settings;
use axum::body::Body;
use axum::http::{HeaderValue, Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use expense_tracker_api::api;
use expense_tracker_db::setup::setup_db;
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, DecodingKey, Validation};
use log::{debug, error, info, warn};
use std::{env};
use std::fs::File;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::LazyLock;
use std::time::Duration;
use clap::Parser;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer, MaxAge};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;
use utoipa::gen::serde_json::Value;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::oauth;
use utoipa_swagger_ui::SwaggerUi;

const SETTINGS_FILE: &str = "config/settings.toml";
static APP_SETTINGS: LazyLock<Settings> =
    LazyLock::new(|| Settings::new(SETTINGS_FILE).expect("Settings file must exist"));

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    export_openapi : bool
}

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
                SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
            )
        }
    }
}

/// Fetches the jwks used for validation of the token
async fn fetch_jwks(jwks_url: &str) -> Result<JwkSet, reqwest::Error> {
    let ignore_tls = env::var("EXPENSE_TRACKER_IGNORE_TLS").is_ok();

    let mut client_builder = reqwest::Client::builder();

    if ignore_tls {
        warn!("TLS has been disabled! Please enable for production use!");
        client_builder = client_builder.danger_accept_invalid_certs(true);
    }

    let client = client_builder.build()?;

    let response = client.get(jwks_url).send().await?;
    let jwks = response.json::<JwkSet>().await?;
    Ok(jwks)
}

async fn validate_token(token: &str, oidc_settings: &settings::Oidc) -> Result<Value, String> {
    debug!("Starting token validation");
    let jwks_url = format!(
        "{}/protocol/openid-connect/certs",
        oidc_settings.issuer_url()
    );
    let jwks = match fetch_jwks(jwks_url.as_str()).await {
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
    validation.set_audience(&[oidc_settings.audience()]);
    validation.set_issuer(&[oidc_settings.issuer_url()]);
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
        .ok_or_else(|| {
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body("Unauthorized, no Bearer token present".to_string())
                .unwrap()
        })?;

    debug!("Token extracted!");

    let claims = validate_token(token, APP_SETTINGS.oidc())
        .await
        .map_err(|e| {
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(format!("Unauthorized, {e}"))
                .unwrap()
        })?;

    // Insert claims into request extensions for use in handlers
    let mut parts = parts;
    parts.extensions.insert(claims);

    let request = Request::from_parts(parts, body);
    Ok(next.run(request).await)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

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

    let pool = setup_db(APP_SETTINGS.expense_tracker().db_connection_string())
        .await
        .expect("Failed to create pool");

    // To get a JWT: curl -X POST 'http://localhost:8080/realms/expense-tracker-dev/protocol/openid-connect/token' -H 'Content-Type: application/x-www-form-urlencoded' -d 'client_id=<CLIENT_ID>' -d 'username=<USER>' -d 'password=<PASSWORD>' -d 'grant_type=password' -d 'scope=email profile' -d 'client_secret=<CLIENT_SECRET>'

    let origins = [
        "http://localhost:3000".parse::<HeaderValue>().unwrap(),
        "http://localhost:3000".parse::<HeaderValue>().unwrap()
    ];

    let cors_layer = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(Any)
        .allow_headers(vec![
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
            http::header::CONTENT_TYPE,
        ])
        .max_age(Duration::from_secs(3600));


    let cors = ServiceBuilder::new()
        .layer(cors_layer);

    let oauth_validator = ServiceBuilder::new()
        .layer(cors.clone())
        .layer(axum::middleware::from_fn(auth_middleware));

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", api::router(pool).await)
        .layer(oauth_validator)
        .nest("/api", api::add_health_api().await)
        .layer(cors)
        // 3. Add a TraceLayer to automatically create and enter spans
        .layer(TraceLayer::new_for_http())
        .split_for_parts();

    // setup oAuth with utoipa swagger ui
    let oauth_config = oauth::Config::new().client_id(APP_SETTINGS.oidc().audience());

    let router = router.merge(
        SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", api.clone())
            .oauth(oauth_config),
    );

    if args.export_openapi {
        let mut file = File::create("./openapi/expense_tracker_openapi.json").expect("Failed to create file");
        file.write_all(api.to_pretty_json().unwrap().as_bytes()).expect("Failed to write to file");
        println!("OpenAPI JSON exported successfully.");
        return;
    }

    let addr = SocketAddr::from(([127, 0, 0, 1], APP_SETTINGS.expense_tracker().port()));

    info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router.into_make_service())
        .await
        .unwrap();
}
