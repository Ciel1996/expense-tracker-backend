extern crate core;

use std::net::SocketAddr;
use axum::body::Body;
use axum::error_handling::HandleErrorLayer;
use axum::http::{Request, StatusCode, Uri};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum_oidc::error::MiddlewareError;
use axum_oidc::{EmptyAdditionalClaims, OidcAuthLayer, OidcLoginLayer, ProviderMetadata};
use cookie::SameSite;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use jsonwebtoken::jwk::JwkSet;
use reqwest::get;
use time::Duration;
use tower::{ServiceBuilder, ServiceExt};
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use utoipa::{Modify, OpenApi};
use utoipa::gen::serde_json::Value;
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;
use utoipa_swagger_ui::oauth;
use expense_tracker_api::api;
use expense_tracker_db::setup::setup_db;
use log::{error, info};

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
    println!("Starting token validation");
    let jwks_url = "http://localhost:8080/realms/expense-tracker-dev/protocol/openid-connect/certs";
    let jwks = match fetch_jwks(jwks_url).await {
        Ok(jwks) => jwks,
        Err(e) => {
            eprintln!("Failed to fetch JWKS: {}", e);
            return Err("Failed to fetch JWKS".to_string());
        }
    };

    let header = match jsonwebtoken::decode_header(token) {
        Ok(header) => header,
        Err(e) => {
            eprintln!("Failed to decode token header: {}", e);
            return Err("Failed to decode token header".to_string());
        }
    };

    let kid = match header.kid {
        Some(kid) => kid,
        None => {
            eprintln!("No kid in header");
            return Err("No kid in header".to_string());
        }
    };

    let key = match jwks.find(&kid) {
        Some(key) => key,
        None => {
            eprintln!("Key not found in JWKS");
            return Err("Key not found in JWKS".to_string());
        }
    };

    let decoding_key = match DecodingKey::from_jwk(key) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Failed to create decoding key: {}", e);
            return Err("Failed to create decoding key".to_string());
        }
    };

    let mut validation = Validation::new(header.alg);
    // TODO: read from config
    validation.set_audience(&["expense-tracker"]);
    match decode::<Value>(token, &decoding_key, &validation) {
        Ok(data) => {
            println!("Token validated successfully");
            Ok(data.claims)
        }
        Err(e) => {
            eprintln!("Failed to decode token: {}", e);
            Err("Failed to decode token".to_string())
        }
    }
}

async fn auth_middleware(request: Request<Body>, next: Next) -> Result<Response, Response<String>> {
    let (parts, body) = request.into_parts();

    let token = parts
        .headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(||
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body("Unauthorized".to_string()).unwrap())?;

    info!("Token extracted: {}", token);

    let claims = validate_token(token)
        .await
        .map_err(|_|
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body("Unauthorized, could not validate token".to_string()).unwrap())?;

    // Insert claims into request extensions for use in handlers
    let mut parts = parts;
    parts.extensions.insert(claims);

    let request = Request::from_parts(parts, body);
    Ok(next.run(request).await)
}

#[tokio::main]
async fn main() {
    let pool = setup_db().await.expect("Failed to create pool");

    let session_store = MemoryStore::default();
    let session_layer =
        SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_same_site(SameSite::Lax)
            .with_expiry(Expiry::OnInactivity(Duration::seconds(60)));

    // let oidc_login_service = ServiceBuilder::new()
    //     .layer(HandleErrorLayer::new(|e: MiddlewareError| async {
    //         e.into_response()
    //     }))
    //     .layer(OidcLoginLayer::<EmptyAdditionalClaims>::new());

    // TODO: read from config
    let uri = Uri::from_maybe_shared("http://localhost:3000/")
        .expect("valid APP_URL");
    let issuer = "http://localhost:8080/realms/expense-tracker-dev".to_string();
    let client_id = "expense-tracker".to_string();
    let client_secret = Some("hZlcMZ9iTuRyKTZISvIfa66Bg9PUrZWk".to_string());
    let scopes = vec![];

    // To get a JWT: curl -X POST 'http://localhost:8080/realms/expense-tracker-dev/protocol/openid-connect/token' -H 'Content-Type: application/x-www-form-urlencoded' -d 'client_id=<CLIENT_ID>' -d 'username=<USER>' -d 'password=<PASSWORD>' -d 'grant_type=password' -d 'scope=email profile' -d 'client_secret=<CLIENT_SECRET>'

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
        )
        .layer(axum::middleware::from_fn(auth_middleware));

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/api", api::router(pool).await)
        // .layer(oidc_login_service)
        .layer(oidc_auth_service)
        .layer(session_layer)
        .split_for_parts();

    let oauth_config = oauth::Config::new()
        .client_id("expense-tracker");

    let router = router
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", api.clone())
            .oauth(oauth_config));

    // TODO: read from config
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router.into_make_service()).await.unwrap();
}
