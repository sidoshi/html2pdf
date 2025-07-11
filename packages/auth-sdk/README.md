# Simple Rust Auth

A minimal Rust library for JWT token validation, inspired by Forto's authentication requirements.

## Features

- ✅ **Simple Token Validation**: Just validates if a token is valid or not
- ✅ **Multiple Issuer Support**: Keycloak, Auth0, and custom SHIP tokens
- ✅ **JWKS Support**: Automatic public key fetching and caching
- ✅ **Symmetric Key Support**: For SHIP tokens using HMAC
- ✅ **Bearer Token Extraction**: Helper for HTTP Authorization headers
- ✅ **Role Extraction**: Extract roles from Keycloak resource_access claims
- ✅ **Minimal Dependencies**: Focused on just what's needed

## Quick Start

```rust
use simple_rust_auth::{TokenValidationConfig, TokenValidator, TokenValidationResult};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the validator
    let config = TokenValidationConfig::new()
        .add_jwks_issuer(
            "https://keycloak-sandbox.forto.com".to_string(),
            "https://keycloak-sandbox.forto.com/auth/realms/tms/protocol/openid-connect/certs".to_string()
        )
        .with_ship_key("your-base64-ship-key".to_string())
        .allow_test_tokens();

    let validator = TokenValidator::new(config);

    // Validate a token
    let token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...";
    
    match validator.validate_token(token).await? {
        TokenValidationResult::Valid { claims } => {
            println!("✅ Token is valid!");
            
            // Get user info
            let user = validator.get_user_from_token(token)?;
            println!("User: {}, Roles: {:?}", user.id, user.roles);
        }
        TokenValidationResult::Expired => {
            println!("❌ Token has expired");
        }
        TokenValidationResult::Invalid { reason } => {
            println!("❌ Token is invalid: {}", reason);
        }
        TokenValidationResult::UnknownIssuer { issuer } => {
            println!("❌ Unknown issuer: {}", issuer);
        }
    }

    Ok(())
}
```

## Configuration

### JWKS Issuers (Keycloak, Auth0)

```rust
let config = TokenValidationConfig::new()
    .add_jwks_issuer(
        "https://your-keycloak.com".to_string(),
        "https://your-keycloak.com/auth/realms/your-realm/protocol/openid-connect/certs".to_string()
    )
    .add_jwks_issuer(
        "https://your-auth0-domain.auth0.com".to_string(),
        "https://your-auth0-domain.auth0.com/.well-known/jwks.json".to_string()
    );
```

### SHIP Symmetric Key

```rust
let config = TokenValidationConfig::new()
    .with_ship_key("your-base64-encoded-symmetric-key".to_string());
```

### Test Tokens (Development)

```rust
let config = TokenValidationConfig::new()
    .allow_test_tokens(); // Allows tokens with "test" in issuer
```

## HTTP Integration

```rust
// Extract from Authorization header
let auth_header = "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...";
let token = validator.extract_token_from_header(auth_header)?;

// Or use the convenience function
use simple_rust_auth::extract_bearer_token;
let token = extract_bearer_token(auth_header)?;
```

## Error Handling

```rust
use simple_rust_auth::{AuthError, Result};

match validator.validate_token(token).await {
    Ok(result) => {
        // Handle validation result
    }
    Err(AuthError::InvalidTokenFormat) => {
        // Handle malformed token
    }
    Err(AuthError::InvalidIssuer { issuer }) => {
        // Handle unknown issuer
    }
    Err(AuthError::RequestError(_)) => {
        // Handle network errors when fetching JWKS
    }
    Err(e) => {
        // Handle other errors
    }
}
```

## Examples

Run the basic example:

```bash
cargo run --example basic_validation
```

## Design Philosophy

This library is intentionally minimal and focused on the single task of token validation. It doesn't include:

- ❌ Token generation/signing
- ❌ Service-to-service client JWTs  
- ❌ Complex authorization policies
- ❌ Middleware integrations

It focuses on:

- ✅ Fast, simple token validation
- ✅ Clear error messages
- ✅ Minimal dependencies
- ✅ Easy configuration

## Dependencies

- `jsonwebtoken` - JWT handling
- `reqwest` - HTTP client for JWKS
- `serde` - JSON serialization
- `tokio` - Async runtime
- `base64` - Base64 decoding
- `chrono` - Time handling
- `lru` - JWKS caching

## License

MIT License
