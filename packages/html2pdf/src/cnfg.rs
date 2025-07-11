use std::sync::Arc;

use dotenv::dotenv;
use once_cell::sync::Lazy;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppEnv {
    #[default]
    Development,
    Production,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Default)]
pub struct AppConfig {
    pub env: AppEnv,

    pub port: u16,
    pub ship_key: String,
    pub jwks_issuers: Vec<(String, String)>,
}

static CONFIG: Lazy<Arc<AppConfig>> = Lazy::new(|| Arc::new(load_config()));

fn extract_issuer_from_jwks_url(jwks_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = Url::parse(jwks_url)?;
    let base_url = format!("{}://{}", url.scheme(), url.host_str().unwrap_or(""));
    Ok(base_url)
}

fn load_config() -> AppConfig {
    dotenv().ok();

    let mut config = AppConfig::default();
    config.port = std::env::var("PORT")
        .unwrap_or_else(|_| 3000.to_string())
        .parse()
        .expect("Invalid PORT value");

    config.ship_key = std::env::var("SHIP_KEY").expect("SHIP_KEY environment variable is not set");

    let auth0_jwks_uri =
        std::env::var("AUTH_JWKS_URI").expect("AUTH_JWKS_URI environment variable is not set");
    let auth_eu_keycloak_jwks_uri = std::env::var("AUTH_EU_KEYCLOAK_JWKS_URI")
        .expect("AUTH_EU_KEYCLOAK_JWKS_URI environment variable is not set");

    config.jwks_issuers.push((
        extract_issuer_from_jwks_url(&auth0_jwks_uri).unwrap(),
        auth0_jwks_uri,
    ));
    config.jwks_issuers.push((
        extract_issuer_from_jwks_url(&auth_eu_keycloak_jwks_uri).unwrap(),
        auth_eu_keycloak_jwks_uri,
    ));

    config
}

pub fn get() -> Arc<AppConfig> {
    Arc::clone(&CONFIG)
}
