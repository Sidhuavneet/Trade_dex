// Solana service module - QuickNode RPC integration

use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone)]
pub struct SolanaService {
    rpc_url: String,
}

#[derive(Debug, Deserialize)]
pub struct RpcResponse<T> {
    pub result: T,
}

#[derive(Debug, Deserialize)]
pub struct SignatureInfo {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
}

impl SolanaService {
    pub fn new() -> Result<Self> {
        let rpc_url = env::var("QUICKNODE_RPC_URL")
            .expect("QUICKNODE_RPC_URL must be set in environment variables");
        
        Ok(Self { rpc_url })
    }


    /// Get transaction details by signature
    pub async fn get_transaction(&self, signature: &str) -> Result<Option<serde_json::Value>> {
        let client = reqwest::Client::new();
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": [
                signature,
                {
                    "encoding": "json",
                    "maxSupportedTransactionVersion": 0
                }
            ]
        });
        
        let response_result = client
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await;
        
        match response_result {
            Ok(response) => {
                let status = response.status();
                
                if status == 429 {
                    return Ok(None); // Return None for rate limit
                }
                
                let raw_text = response.text().await?;
                let json_result: serde_json::Value = serde_json::from_str(&raw_text)?;
                
                if let Some(result) = json_result.get("result") {
                    if result.is_null() {
                        Ok(None)
                    } else {
                        // println!("get_transaction: {} - Transaction found", &signature);
                        Ok(Some(result.clone()))
                    }
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                Err(anyhow::anyhow!("HTTP request failed: {}", e))
            }
        }
    }

}

