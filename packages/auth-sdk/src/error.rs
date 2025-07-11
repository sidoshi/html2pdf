use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid token format")]
    InvalidTokenFormat,
    
    #[error("Token has expired")]
    TokenExpired,
    
    #[error("Invalid issuer: {issuer}")]
    InvalidIssuer { issuer: String },
    
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    
    #[error("HTTP request error: {0}")]
    RequestError(#[from] reqwest::Error),
    
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Missing configuration for issuer")]
    MissingConfig,
    
    #[error("Invalid symmetric key")]
    InvalidSymmetricKey,
}

pub type Result<T> = std::result::Result<T, AuthError>;
