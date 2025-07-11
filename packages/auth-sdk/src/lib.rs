pub mod config;
pub mod error;
pub mod models;
pub mod validator;

pub use config::TokenValidationConfig;
pub use error::{AuthError, Result};
pub use models::{Claims, User, TokenValidationResult};
pub use validator::TokenValidator;

/// Convenience function to extract token from Authorization header
pub fn extract_bearer_token(authorization_header: &str) -> Result<String> {
    if !authorization_header.starts_with("Bearer ") {
        return Err(AuthError::InvalidTokenFormat);
    }
    
    let token = authorization_header.strip_prefix("Bearer ").unwrap().trim();
    if token.is_empty() {
        return Err(AuthError::InvalidTokenFormat);
    }
    
    Ok(token.to_string())
}
