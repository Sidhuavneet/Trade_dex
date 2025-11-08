// Authentication model module

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyRequest {
    pub public_key: String,
    pub signature: String,
    pub nonce: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResponse {
    pub token: String,
    pub expires_at: String,
}

