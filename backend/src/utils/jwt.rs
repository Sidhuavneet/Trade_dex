// JWT utility module

use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

// JWT secret key (in production, use environment variable)
const JWT_SECRET: &str = "your-secret-key-change-in-production";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (public key)
    pub exp: usize, // Expiration time
    pub iat: usize, // Issued at
}

pub fn generate_token(public_key: &str) -> Result<(String, String), anyhow::Error> {
    let now = Utc::now();
    let expires_at = now + Duration::hours(24); // 24 hour expiry
    
    let claims = Claims {
        sub: public_key.to_string(),
        exp: expires_at.timestamp() as usize,
        iat: now.timestamp() as usize,
    };
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )?;
    
    Ok((token, expires_at.to_rfc3339()))
}

