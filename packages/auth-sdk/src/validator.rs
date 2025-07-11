use crate::{config::TokenValidationConfig, error::*, models::*};
use base64::{engine::general_purpose, Engine as _};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use lru::LruCache;
use reqwest::Client;
use serde_json::Value;
use std::{
    collections::HashMap,
    num::NonZeroUsize,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

/// Simple token validator
pub struct TokenValidator {
    config: TokenValidationConfig,
    http_client: Client,
    jwks_cache: Mutex<LruCache<String, HashMap<String, DecodingKey>>>,
}

impl TokenValidator {
    /// Create a new token validator
    pub fn new(config: TokenValidationConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
            jwks_cache: Mutex::new(LruCache::new(NonZeroUsize::new(10).unwrap())),
        }
    }

    /// Extract token from Authorization header
    pub fn extract_token_from_header(&self, authorization_header: &str) -> Result<String> {
        if !authorization_header.starts_with("Bearer ") {
            return Err(AuthError::InvalidTokenFormat);
        }

        let token = authorization_header.strip_prefix("Bearer ").unwrap().trim();
        if token.is_empty() {
            return Err(AuthError::InvalidTokenFormat);
        }

        Ok(token.to_string())
    }

    /// Validate a JWT token
    pub async fn validate_token(&self, token: &str) -> Result<TokenValidationResult> {
        // First decode without validation to get issuer info
        let header = decode_header(token).map_err(AuthError::JwtError)?;
        let claims = self.decode_token_unsafe(token)?;

        // Check expiration first
        if let Some(exp) = claims.exp {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            if now >= exp {
                return Ok(TokenValidationResult::Expired);
            }
        }

        // Check for SHIP tokens first (they don't have an issuer claim)
        if claims.token_requested_from.is_some()
            || claims.user_id.is_some()
            || claims.customer_id.is_some()
        {
            return self.validate_ship_token(token, &claims).await;
        }

        // Get issuer (required for non-SHIP tokens)
        let issuer = claims.iss.as_ref().ok_or(AuthError::InvalidTokenFormat)?;

        // Check for test tokens
        if self.config.allow_test_tokens && issuer.contains("test") {
            return Ok(TokenValidationResult::Valid { claims });
        }

        // Validate with JWKS
        self.validate_jwks_token(token, issuer, &header, &claims)
            .await
    }

    /// Get user from validated token
    pub fn get_user_from_token(&self, token: &str) -> Result<User> {
        let claims = self.decode_token_unsafe(token)?;
        Ok(User::from_claims(&claims))
    }

    // Private helper methods

    /// Decode token without validation (unsafe - only for extracting claims)
    fn decode_token_unsafe(&self, token: &str) -> Result<Claims> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AuthError::InvalidTokenFormat);
        }

        // Decode payload
        let payload = general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|_| AuthError::InvalidTokenFormat)?;

        let claims: Claims = serde_json::from_slice(&payload)?;
        Ok(claims)
    }

    /// Validate SHIP symmetric key token
    async fn validate_ship_token(
        &self,
        token: &str,
        _claims: &Claims,
    ) -> Result<TokenValidationResult> {
        let ship_key = self
            .config
            .ship_symmetric_key
            .as_ref()
            .ok_or(AuthError::MissingConfig)?;

        // SHIP tokens don't have standard JWT claims like issuer, audience, etc.
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims.clear(); // Don't require standard claims
        validation.validate_exp = true; // But do validate expiration
        validation.validate_aud = false;
        validation.validate_nbf = false;

        let decoding_key = DecodingKey::from_secret(ship_key.as_bytes());

        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => {
                return Ok(TokenValidationResult::Valid {
                    claims: token_data.claims,
                });
            }
            Err(e) => match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    return Ok(TokenValidationResult::Expired);
                }
                _ => (),
            },
        }

        Ok(TokenValidationResult::Invalid {
            reason: format!("All key formats failed"),
        })
    }

    /// Validate JWKS token (Keycloak, Auth0, etc.)
    async fn validate_jwks_token(
        &self,
        token: &str,
        issuer: &str,
        header: &jsonwebtoken::Header,
        _claims: &Claims,
    ) -> Result<TokenValidationResult> {
        // Find JWKS URL for this issuer

        dbg!(&self.config.jwks_issuers);

        let jwks_url = self
            .config
            .jwks_issuers
            .iter()
            .find(|(iss, _)| issuer.contains(iss.as_str()))
            .map(|(_, url)| url)
            .ok_or_else(|| AuthError::InvalidIssuer {
                issuer: issuer.to_string(),
            })?;

        // Get kid from header
        let kid = header.kid.as_ref().ok_or(AuthError::InvalidTokenFormat)?;

        // Get decoding key from JWKS
        let decoding_key = self.get_decoding_key_from_jwks(jwks_url, kid).await?;

        // Validate token
        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[issuer]);
        validation.validate_aud = false; // Don't validate audience for JWKS tokens

        match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => Ok(TokenValidationResult::Valid {
                claims: token_data.claims,
            }),
            Err(e) => match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    Ok(TokenValidationResult::Expired)
                }
                _ => Ok(TokenValidationResult::Invalid {
                    reason: e.to_string(),
                }),
            },
        }
    }

    /// Get decoding key from JWKS
    async fn get_decoding_key_from_jwks(&self, jwks_url: &str, kid: &str) -> Result<DecodingKey> {
        // Check cache first
        {
            let mut cache = self.jwks_cache.lock().unwrap();
            if let Some(keys) = cache.get(jwks_url) {
                if let Some(key) = keys.get(kid) {
                    return Ok(key.clone());
                }
            }
        }

        // Fetch JWKS
        let jwks: Value = self.http_client.get(jwks_url).send().await?.json().await?;

        let keys = jwks["keys"]
            .as_array()
            .ok_or(AuthError::InvalidTokenFormat)?;

        let mut decoded_keys = HashMap::new();

        for key in keys {
            let key_kid = key["kid"].as_str().unwrap_or("");
            let kty = key["kty"].as_str().unwrap_or("");

            if kty == "RSA" {
                let n = key["n"].as_str().ok_or(AuthError::InvalidTokenFormat)?;
                let e = key["e"].as_str().ok_or(AuthError::InvalidTokenFormat)?;

                if let Ok(decoding_key) = DecodingKey::from_rsa_components(n, e) {
                    decoded_keys.insert(key_kid.to_string(), decoding_key);
                }
            }
        }

        // Update cache
        {
            let mut cache = self.jwks_cache.lock().unwrap();
            cache.put(jwks_url.to_string(), decoded_keys.clone());
        }

        decoded_keys
            .get(kid)
            .cloned()
            .ok_or(AuthError::InvalidTokenFormat)
    }
}
