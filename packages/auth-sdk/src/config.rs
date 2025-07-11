use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Simple token validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidationConfig {
    /// List of valid issuers and their JWKS URLs
    pub jwks_issuers: HashMap<String, String>,
    /// SHIP symmetric key (base64 encoded)
    pub ship_symmetric_key: Option<String>,
    /// Whether to allow test tokens (for development)
    pub allow_test_tokens: bool,
}

impl Default for TokenValidationConfig {
    fn default() -> Self {
        Self {
            jwks_issuers: HashMap::new(),
            ship_symmetric_key: None,
            allow_test_tokens: false,
        }
    }
}

impl TokenValidationConfig {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a JWKS issuer (for Keycloak, Auth0, etc.)
    pub fn add_jwks_issuer(mut self, issuer: String, jwks_url: String) -> Self {
        self.jwks_issuers.insert(issuer, jwks_url);
        self
    }
    
    /// Set SHIP symmetric key
    pub fn with_ship_key(mut self, key: String) -> Self {
        self.ship_symmetric_key = Some(key);
        self
    }
    
    /// Enable test tokens
    pub fn allow_test_tokens(mut self) -> Self {
        self.allow_test_tokens = true;
        self
    }
}
