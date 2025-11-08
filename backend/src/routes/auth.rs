// Auth routes module

use axum::{routing::{get, post}, Router, Json, extract::State};
use serde_json::json;
use rand::{distributions::Alphanumeric, Rng};
use ed25519_dalek::{VerifyingKey, Signature};
use bs58;
use crate::models::auth::{VerifyRequest, VerifyResponse};
use crate::utils::jwt;
use crate::state::AppState;

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

async fn get_nonce() -> Json<serde_json::Value> {
    // Generate a random 32-character alphanumeric nonce
    let nonce: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    
    Json(json!({ "nonce": nonce }))
}

async fn verify_signature(
    State(state): State<std::sync::Arc<AppState>>,
    Json(payload): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, axum::response::Json<serde_json::Value>> {
    // Step 1: Decode public key from base58
    let public_key_bytes = bs58::decode(&payload.public_key)
        .into_vec()
        .map_err(|e| {
            axum::response::Json(json!({
                "error": "Invalid public key",
                "message": format!("Failed to decode public key: {}", e)
            }))
        })?;
    
    // Step 2: Decode signature from base58
    let signature_bytes = bs58::decode(&payload.signature)
        .into_vec()
        .map_err(|e| {
            axum::response::Json(json!({
                "error": "Invalid signature",
                "message": format!("Failed to decode signature: {}", e)
            }))
        })?;
    
    // Step 3: Create the message that was signed
    let message = format!("Sign this message to authenticate with Trade: {}", payload.nonce);
    let message_bytes = message.as_bytes();
    
    // Step 4: Verify the signature
    let verifying_key = VerifyingKey::from_bytes(
        public_key_bytes[..32].try_into().map_err(|_| {
            axum::response::Json(json!({
                "error": "Invalid public key",
                "message": "Public key must be 32 bytes"
            }))
        })?
    ).map_err(|e| {
        axum::response::Json(json!({
            "error": "Invalid public key",
            "message": format!("Failed to create verifying key: {}", e)
        }))
    })?;
    
    // Convert signature bytes to fixed-size array
    let signature_array: [u8; 64] = signature_bytes[..64].try_into().map_err(|_| {
        axum::response::Json(json!({
            "error": "Invalid signature",
            "message": "Signature must be 64 bytes"
        }))
    })?;
    
    let signature = Signature::from_bytes(&signature_array);
    
    // Step 5: Verify signature
    verifying_key.verify_strict(message_bytes, &signature)
        .map_err(|e| {
            axum::response::Json(json!({
                "error": "Signature verification failed",
                "message": format!("Signature is invalid: {}", e)
            }))
        })?;
    
    // Step 6: Generate JWT token
    let (token, expires_at) = jwt::generate_token(&payload.public_key)
        .map_err(|e| {
            axum::response::Json(json!({
                "error": "Token generation failed",
                "message": format!("Failed to generate token: {}", e)
            }))
        })?;
    
    // Step 7: Store session in ClickHouse
    let expires_at_dt = chrono::DateTime::parse_from_rfc3339(&expires_at)
        .map_err(|_| {
            axum::response::Json(json!({
                "error": "Invalid expiry date",
                "message": "Failed to parse expiry date"
            }))
        })?
        .with_timezone(&chrono::Utc);
    
    if let Err(e) = state.clickhouse.store_session(&payload.public_key, &token, expires_at_dt).await {
        eprintln!("❌ Failed to store session in ClickHouse: {}", e);
        eprintln!("   User: {}, Token: {}...", payload.public_key, &token[..20.min(token.len())]);
        // Continue even if session storage fails (auth still succeeds)
    } else {
        println!("✅ Stored session in ClickHouse for user: {}", payload.public_key);
    }
    
    Ok(Json(VerifyResponse {
        token,
        expires_at,
    }))
}

pub fn routes() -> Router<std::sync::Arc<crate::state::AppState>> {
    Router::new()
        .route("/health", get(health))
        .route("/nonce", get(get_nonce))
        .route("/verify", post(verify_signature))
}

