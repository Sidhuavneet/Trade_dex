// QuickNode WebSocket subscription service for real-time trade ingestion
// Uses logsSubscribe to monitor DEX program logs for swap transactions

use crate::models::trade::Trade;
use crate::services::solana::SolanaService;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use futures_util::{SinkExt, StreamExt};
use url::Url;

#[derive(Clone)]
pub struct QuickNodeWebSocket {
    rpc_url: String,
    solana_service: Arc<SolanaService>,
}

// JSON-RPC notification wrapper
#[derive(Debug, Deserialize)]
struct JsonRpcNotification {
    #[serde(default)]
    pub jsonrpc: String,
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub id: Option<u64>, // Present in responses, absent in notifications
    #[serde(default)]
    pub params: Option<LogNotificationParams>, // Optional for subscription confirmations
    #[serde(default)]
    pub result: Option<serde_json::Value>, // Present in subscription confirmations
}

// Log notification params (what's inside the params field)
#[derive(Debug, Deserialize)]
struct LogNotificationParams {
    pub subscription: u64,
    pub result: LogResult,
}

#[derive(Debug, Deserialize)]
struct LogResult {
    pub context: LogContext,
    pub value: LogValue,
}

#[derive(Debug, Deserialize)]
struct LogContext {
    pub slot: u64,
}

#[derive(Debug, Deserialize)]
struct LogValue {
    pub signature: String,
    pub err: Option<serde_json::Value>,
    pub logs: Vec<String>,
}

// Transaction data structures
#[derive(Debug, Deserialize)]
struct TransactionData {
    pub slot: u64,
    #[serde(rename = "blockTime")]
    pub block_time: Option<i64>,
    pub meta: Option<TransactionMeta>,
    pub transaction: TransactionInfo,
}

#[derive(Debug, Deserialize)]
struct TransactionInfo {
    pub message: TransactionMessage,
    pub signatures: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TransactionMessage {
    #[serde(rename = "accountKeys")]
    #[serde(default)]
    pub account_keys: Vec<serde_json::Value>, // Can be strings or objects
    #[serde(default)]
    pub instructions: Vec<serde_json::Value>,
    #[serde(rename = "recentBlockhash")]
    #[serde(default)]
    pub recent_blockhash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TransactionMeta {
    #[serde(rename = "preTokenBalances")]
    pub pre_token_balances: Option<Vec<TokenBalance>>,
    #[serde(rename = "postTokenBalances")]
    pub post_token_balances: Option<Vec<TokenBalance>>,
    #[serde(rename = "preBalances")]
    pub pre_balances: Option<Vec<u64>>,
    #[serde(rename = "postBalances")]
    pub post_balances: Option<Vec<u64>>,
    #[serde(rename = "logMessages")]
    pub log_messages: Option<Vec<String>>,
    pub err: Option<serde_json::Value>,
    pub fee: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TokenBalance {
    #[serde(rename = "accountIndex")]
    pub account_index: u8,
    pub mint: String,
    #[serde(rename = "uiTokenAmount")]
    pub ui_token_amount: Option<TokenAmount>,
}

#[derive(Debug, Deserialize)]
struct TokenAmount {
    #[serde(rename = "uiAmount")]
    pub ui_amount: Option<f64>,
}

#[derive(Debug, Serialize)]
struct SubscribeRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<serde_json::Value>,
}

impl QuickNodeWebSocket {
    pub fn new(solana_service: Arc<SolanaService>) -> Result<Self> {
        let rpc_url = std::env::var("QUICKNODE_RPC_URL")
            .context("QUICKNODE_RPC_URL must be set")?;
        
        Ok(Self {
            rpc_url,
            solana_service,
        })
    }

    /// Start WebSocket subscription to DEX program logs
    /// Returns a channel receiver for trade updates
    pub async fn start_subscription(
        &self,
        trade_tx: mpsc::Sender<Trade>,
    ) -> Result<()> {
        // Convert HTTP URL to WebSocket URL
        let ws_url = self.rpc_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");
        
        let url = Url::parse(&ws_url)
            .context("Invalid QuickNode WebSocket URL")?;
        
        let (ws_stream, _) = connect_async(url)
            .await
            .context("Failed to connect to QuickNode WebSocket")?;
        
        let (mut write, mut read) = ws_stream.split();
        
        let dex_programs = vec![
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4", // Jupiter v6
            "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB", // Jupiter v4
            "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8", // Raydium
            "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP", // Orca
            "9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp", // Meteora
            "PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLRJi5i4Z2j3Yc", // Phoenix
        ];
        
        // Subscribe to logs for each DEX program
        for (idx, program_id) in dex_programs.iter().enumerate() {
            let subscribe_req = SubscribeRequest {
                jsonrpc: "2.0".to_string(),
                id: idx as u64 + 1,
                method: "logsSubscribe".to_string(),
                params: vec![
                    json!({
                        "mentions": [program_id]
                    }),
                    json!({
                        "commitment": "confirmed"
                    }),
                ],
            };
            
            let msg = serde_json::to_string(&subscribe_req)?;
            write.send(WsMessage::Text(msg)).await?;
        }
        
        // Process incoming messages
        let solana_clone = self.solana_service.clone();
        let mut seen_signatures = std::collections::HashSet::new();
        
        while let Some(msg) = read.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    // Try to parse as JSON-RPC notification
                    if let Ok(jsonrpc_notif) = serde_json::from_str::<JsonRpcNotification>(&text) {
                        // Handle subscription confirmation responses
                        if jsonrpc_notif.id.is_some() && jsonrpc_notif.result.is_some() && jsonrpc_notif.method.is_empty() {
                            continue;
                        }
                        
                        // Check if it's a logsNotification
                        if jsonrpc_notif.method == "logsNotification" {
                            let log_notif = match jsonrpc_notif.params {
                                Some(params) => params,
                                None => continue,
                            };
                            
                            let signature = log_notif.result.value.signature.clone();
                            
                            // Deduplicate by signature
                            if seen_signatures.contains(&signature) {
                                continue;
                            }
                            seen_signatures.insert(signature.clone());
                            
                            // Keep only last 1000 signatures to prevent memory leak
                            if seen_signatures.len() > 1000 {
                                seen_signatures.clear();
                            }
                            
                            // Skip failed transactions (ONLY rejection criteria)
                            if log_notif.result.value.err.is_some() {
                                continue;
                            }
                            
                            // Commented out: Check if logs contain swap indicators
                            // let is_swap = Self::is_swap_transaction(&log_notif.result.value.logs);
                            
                            // Fetch full transaction details for all successful transactions
                            // (Previously only fetched if is_swap was true)
                            // if is_swap {
                                // Fetch full transaction details
                                let solana_clone = solana_clone.clone();
                                let signature_clone = signature.clone();
                                let slot_clone = log_notif.result.context.slot;
                                let trade_tx_clone = trade_tx.clone();
                                
                                tokio::spawn(async move {
                                    match solana_clone.get_transaction(&signature_clone).await {
                                        Ok(Some(tx_json)) => {
                                            // Parse transaction data
                                            if let Ok(tx_data) = serde_json::from_value::<TransactionData>(tx_json) {
                                                // Construct trade from both logsSubscribe and getTransaction data
                                                if let Some(trade) = Self::construct_trade(
                                                    &signature_clone,
                                                    &slot_clone,
                                                    &tx_data,
                                                ) {
                                                    if let Err(_) = trade_tx_clone.send(trade).await {
                                                        // Channel closed, ignore
                                                    }
                                                }
                                            }
                                        }
                                        Ok(None) => {}
                                        Err(_) => {}
                                    }
                                });
                            // }
                        }
                    }
                }
                Ok(WsMessage::Close(_)) => {
                    break;
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Check if transaction logs indicate a swap
    fn is_swap_transaction(logs: &[String]) -> bool {
        // Look for swap-related log messages
        // Jupiter uses "Instruction: Route" for swaps
        // Raydium uses "Instruction: Swap" variants
        let swap_keywords = vec![
            "Instruction: Swap",
            "Instruction: SwapBaseIn",
            "Instruction: SwapBaseOut",
            "Instruction: Route", // Jupiter swap route
        ];
        
        // Check for swap keywords in logs
        logs.iter().any(|log| {
            swap_keywords.iter().any(|keyword| log.contains(keyword))
        })
    }
    
    /// Construct trade from logsSubscribe and getTransaction data
    fn construct_trade(
        signature: &str,
        slot: &u64,
        tx_data: &TransactionData,
    ) -> Option<Trade> {
        // Commented out: Check if meta exists (use default if None)
        // let meta = tx_data.meta.as_ref()?;
        let meta = match tx_data.meta.as_ref() {
            Some(m) => m,
            None => return None, // Still need meta for trade construction
        };
        
        // Commented out: Check if transaction succeeded (already checked in logsSubscribe)
        // if meta.err.is_some() {
        //     return None;
        // }
        
        // Get pre and post token balances
        // Commented out: Check if balances exist (use empty vec if None)
        // let pre_balances = meta.pre_token_balances.as_ref()?;
        // let post_balances = meta.post_token_balances.as_ref()?;
        
        // Commented out: Check if balances are empty
        // if pre_balances.is_empty() || post_balances.is_empty() {
        //     return None;
        // }
        
        // Use empty vec if None to continue processing
        let empty_pre: Vec<TokenBalance> = vec![];
        let empty_post: Vec<TokenBalance> = vec![];
        let pre_balances = meta.pre_token_balances.as_ref().unwrap_or(&empty_pre);
        let post_balances = meta.post_token_balances.as_ref().unwrap_or(&empty_post);
        
        // Find largest deltas to determine base and quote mints
        let mut mint_deltas: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        
        // Create maps for easier lookup
        let mut pre_map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        let mut post_map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        
        for balance in pre_balances.iter() {
            let amount = balance.ui_token_amount.as_ref()
                .and_then(|t| t.ui_amount)
                .unwrap_or(0.0);
            *pre_map.entry(balance.mint.clone()).or_insert(0.0) += amount;
        }
        
        for balance in post_balances.iter() {
            let amount = balance.ui_token_amount.as_ref()
                .and_then(|t| t.ui_amount)
                .unwrap_or(0.0);
            *post_map.entry(balance.mint.clone()).or_insert(0.0) += amount;
        }
        
        // Calculate deltas for each mint
        let mut all_mints = std::collections::HashSet::new();
        for mint in pre_map.keys() {
            all_mints.insert(mint.clone());
        }
        for mint in post_map.keys() {
            all_mints.insert(mint.clone());
        }
        
        for mint in all_mints {
            let pre = pre_map.get(&mint).copied().unwrap_or(0.0);
            let post = post_map.get(&mint).copied().unwrap_or(0.0);
            let delta = (post - pre).abs();
            if delta > 0.0 {
                mint_deltas.insert(mint, delta);
            }
        }
        
        // Find the two mints with largest deltas
        let mut sorted_deltas: Vec<_> = mint_deltas.iter().collect();
        sorted_deltas.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Commented out: Check if we have at least 2 mints
        // if sorted_deltas.len() < 2 {
        //     return None;
        // }
        
        // Use first two mints, or use placeholder if less than 2
        let base_mint = sorted_deltas.get(0).map(|(mint, _)| (*mint).clone()).unwrap_or_else(|| "UNKNOWN".to_string());
        let quote_mint = sorted_deltas.get(1).map(|(mint, _)| (*mint).clone()).unwrap_or_else(|| "UNKNOWN".to_string());
        
        // Calculate amounts
        let pre_base = pre_map.get(base_mint.as_str()).copied().unwrap_or(0.0);
        let post_base = post_map.get(base_mint.as_str()).copied().unwrap_or(0.0);
        let base_amount = (post_base - pre_base).abs();
        
        let pre_quote = pre_map.get(quote_mint.as_str()).copied().unwrap_or(0.0);
        let post_quote = post_map.get(quote_mint.as_str()).copied().unwrap_or(0.0);
        let quote_amount = (post_quote - pre_quote).abs();
        
        // Commented out: Check if amounts are too small
        // if base_amount < 0.000001 || quote_amount < 0.0001 {
        //     return None;
        // }
        
        // Filter: Only process trades involving allowed tokens
        // Both base and quote mints must be in the allowed list
        if !Self::is_allowed_mint(&base_mint) || !Self::is_allowed_mint(&quote_mint) {
            return None; // Reject trades with unknown tokens
        }
        
        // Map mints to symbols
        let base_symbol = Self::mint_to_symbol(&base_mint);
        let quote_symbol = Self::mint_to_symbol(&quote_mint);
        
        // Use symbols (should never be UNKNOWN now due to filtering above)
        let final_base_symbol = base_symbol;
        let final_quote_symbol = quote_symbol;
        
        // Calculate price (handle division by zero)
        let final_price = if base_amount > 0.0 {
            quote_amount / base_amount
        } else {
            0.0
        };
        
        // Determine side based on base delta
        let base_delta = post_base - pre_base;
        let side = if base_delta > 0.0 { "buy" } else { "sell" };
        
        // Calculate total value
        let total_value = final_price * base_amount;
        
        // Identify DEX program
        let dex_program = meta.log_messages.as_ref()
            .map(|logs| {
                let logs_str = logs.join(" ");
                if logs_str.contains("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4") {
                    "Jupiter v6"
                } else if logs_str.contains("JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB") {
                    "Jupiter v4"
                } else if logs_str.contains("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8") {
                    "Raydium"
                } else if logs_str.contains("9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP") {
                    "Orca"
                } else if logs_str.contains("9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp") {
                    "Meteora"
                } else if logs_str.contains("PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLRJi5i4Z2j3Yc") {
                    "Phoenix"
                } else {
                    "Unknown"
                }
            })
            .unwrap_or("Unknown");
        
        // Get timestamp
        let block_time = tx_data.block_time.unwrap_or(Utc::now().timestamp());
        
        Some(Trade {
            id: signature.to_string(),
            timestamp: chrono::DateTime::from_timestamp(block_time, 0)
                .unwrap_or_else(|| Utc::now()),
            base_symbol: final_base_symbol,
            quote_symbol: final_quote_symbol,
            base_mint: base_mint.clone(),
            quote_mint: quote_mint.clone(),
            price: final_price,
            amount: base_amount,
            side: side.to_string(),
            total_value,
            dex_program: dex_program.to_string(),
            slot: *slot,
        })
    }
    
    /// Map mint address to symbol
    fn mint_to_symbol(mint: &str) -> String {
        match mint {
            "So11111111111111111111111111111111111111112" => "SOL".to_string(),
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC".to_string(),
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => "USDT".to_string(),
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" => "BONK".to_string(),
            "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN" => "JUP".to_string(),
            "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm" => "WIF".to_string(),
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" => "RAY".to_string(),
            _ => "UNKNOWN".to_string(),
        }
    }
    
    /// Check if mint is in the allowed list
    fn is_allowed_mint(mint: &str) -> bool {
        matches!(
            mint,
            "So11111111111111111111111111111111111111112" | // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" | // USDC
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" | // USDT
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" | // BONK
            "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN" | // JUP
            "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm" | // WIF
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R"   // RAY
        )
    }
}

