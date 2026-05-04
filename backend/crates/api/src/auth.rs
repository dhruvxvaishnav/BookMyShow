use chrono::Utc;
use common::AppError;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // user_id
    pub email: String,
    pub user_name: String,
    pub role: String,       // "user" or "admin"
    pub token_type: String, // "access" or "refresh"
    pub iat: usize,
    pub exp: usize,
}

pub fn encode_access_token(
    user_id: &str,
    email: &str,
    user_name: &str,
    role: &str,
    secret: &str,
    expiry_secs: u64,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        user_name: user_name.to_string(),
        role: role.to_string(),
        token_type: "access".to_string(),
        iat: now,
        exp: now + expiry_secs as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalError(format!("JWT encode: {e}")))
}

pub fn encode_refresh_token(
    user_id: &str,
    email: &str,
    user_name: &str,
    role: &str,
    secret: &str,
    expiry_secs: u64,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        user_name: user_name.to_string(),
        role: role.to_string(),
        token_type: "refresh".to_string(),
        iat: now,
        exp: now + expiry_secs as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalError(format!("JWT encode: {e}")))
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}
