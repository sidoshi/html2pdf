use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

// Custom deserializer for timestamps that can be either int or float
fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    match value {
        Some(serde_json::Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                Ok(Some(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Some(f as i64))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

/// JWT Claims extracted from token
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub iss: Option<String>,            // Issuer
    pub sub: Option<String>,            // Subject (user ID)
    pub aud: Option<serde_json::Value>, // Audience (can be string or array)
    pub azp: Option<String>,            // Authorized party
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub exp: Option<i64>, // Expiration time
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub iat: Option<i64>, // Issued at
    pub email: Option<String>,          // Email
    pub name: Option<String>,           // Name
    pub resource_access: Option<serde_json::Value>, // Keycloak roles

    // SHIP-specific fields
    #[serde(rename = "customerId")]
    pub customer_id: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "tokenRequestedFrom")]
    pub token_requested_from: Option<String>,

    #[serde(flatten)]
    pub additional_claims: HashMap<String, serde_json::Value>,
}

/// User information extracted from JWT
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub roles: Vec<String>,
}

impl User {
    /// Extract user from claims
    pub fn from_claims(claims: &Claims) -> Self {
        let roles = extract_roles_from_claims(claims);

        // For SHIP tokens, prefer user_id over sub
        let id = claims
            .user_id
            .clone()
            .or_else(|| claims.sub.clone())
            .unwrap_or_default();

        Self {
            id,
            email: claims.email.clone(),
            name: claims.name.clone(),
            roles,
        }
    }
}

/// Extract roles from Keycloak resource_access claim
fn extract_roles_from_claims(claims: &Claims) -> Vec<String> {
    let mut roles = Vec::new();

    if let Some(resource_access) = &claims.resource_access {
        // Try TMS client
        if let Some(tms) = resource_access.get("tms") {
            if let Some(tms_roles) = tms.get("roles") {
                if let Some(roles_array) = tms_roles.as_array() {
                    for role in roles_array {
                        if let Some(role_str) = role.as_str() {
                            roles.push(role_str.to_string());
                        }
                    }
                }
            }
        }

        // Try other clients if TMS doesn't exist
        if roles.is_empty() {
            if let Some(obj) = resource_access.as_object() {
                for (_client, client_data) in obj {
                    if let Some(client_roles) = client_data.get("roles") {
                        if let Some(roles_array) = client_roles.as_array() {
                            for role in roles_array {
                                if let Some(role_str) = role.as_str() {
                                    roles.push(role_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    roles
}

/// Token validation result
#[derive(Debug)]
pub enum TokenValidationResult {
    Valid { claims: Claims },
    Invalid { reason: String },
    Expired,
    UnknownIssuer { issuer: String },
}
