mod browser_pool;
mod cnfg;
mod error;
mod html2pdf;

use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Request, State};
use axum::http::Method;
use axum::middleware;
use axum::response::Response;
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};

use auth_sdk::{TokenValidationConfig, TokenValidator};
use browser_pool::BrowserPool;
use html2pdf::html2pdf;

async fn auth_middleware(
    State(app_state): State<AppState>,
    request: Request,
    next: middleware::Next,
) -> Response {
    let auth_header = request.headers().get("Authorization");

    if auth_header.is_none() {
        return Response::builder()
            .status(axum::http::StatusCode::UNAUTHORIZED)
            .body("Missing Authorization header".into())
            .unwrap();
    }

    let token_validator = &app_state.token_validator;
    let token =
        token_validator.extract_token_from_header(auth_header.unwrap().to_str().unwrap_or(""));
    if token.is_err() {
        return Response::builder()
            .status(axum::http::StatusCode::UNAUTHORIZED)
            .body("Invalid token".into())
            .unwrap();
    }

    let token = token.unwrap();
    match token_validator.validate_token(&token).await {
        Ok(_) => {
            // Your middleware logic here
            next.run(request).await
        }
        Err(e) => {
            return Response::builder()
                .status(axum::http::StatusCode::UNAUTHORIZED)
                .body(format!("Token validation failed: {}", e).into())
                .unwrap();
        }
    }
}

#[derive(Clone)]
struct AppState {
    browser_pool: Arc<BrowserPool>,
    token_validator: Arc<TokenValidator>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = cnfg::get();
    tracing_subscriber::fmt::init();

    let browser_pool = Arc::new(BrowserPool::new().await?);
    let mut token_validator_config =
        TokenValidationConfig::new().with_ship_key(config.ship_key.clone());

    for jwks_issuer in &config.jwks_issuers {
        token_validator_config =
            token_validator_config.add_jwks_issuer(jwks_issuer.0.clone(), jwks_issuer.1.clone());
    }
    let token_validator = TokenValidator::new(token_validator_config);

    let app_state = AppState {
        browser_pool: browser_pool,
        token_validator: Arc::new(token_validator),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .allow_credentials(false);

    let protected_routes = Router::new()
        .route("/html2pdf", post(html2pdf))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ))
        .with_state(app_state);

    let app = Router::new()
        .merge(protected_routes)
        .route("/healthz", get(healthz))
        .layer(cors);

    let port = config.port;
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn healthz() -> &'static str {
    "Pong"
}
