// Trades routes module

use axum::{routing::get, Router, Json, extract::State};
use serde_json::json;
use crate::state::AppState;
use std::collections::HashMap;

/// Get recent trades filtered by pair (from ClickHouse)
async fn get_trades(
    State(state): State<std::sync::Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, axum::response::Json<serde_json::Value>> {
    println!("📥 GET /api/trades - Request received");
    println!("   Query params: {:?}", params);
    
    let pair = params.get("pair").cloned().unwrap_or_else(|| "SOL/USDC".to_string());
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(100);
    
    println!("   Parsed pair: {}, limit: {}", pair, limit);

    // Parse pair
    let parts: Vec<&str> = pair.split('/').collect();
    if parts.len() != 2 {
        return Err(axum::response::Json(json!({
            "error": "Invalid pair format",
            "message": "Pair must be in format BASE/QUOTE"
        })));
    }

    let base_symbol = parts[0];
    let quote_symbol = parts[1];
    
    println!("   Querying ClickHouse for {}/{} (limit: {})", base_symbol, quote_symbol, limit);

    // Query ClickHouse for trades
    match state.clickhouse.get_trades(base_symbol, quote_symbol, limit).await {
        Ok(trades) => {
            println!("✅ Successfully fetched {} trades from ClickHouse", trades.len());
            Ok(Json(json!(trades)))
        },
        Err(e) => {
            eprintln!("❌ ClickHouse query error for {}/{}: {}", base_symbol, quote_symbol, e);
            Err(axum::response::Json(json!({
                "error": "Failed to query trades",
                "message": format!("{}", e)
            })))
        }
    }
}

/// Get OHLCV data for a pair and interval (from ClickHouse)
async fn get_ohlcv(
    State(state): State<std::sync::Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, axum::response::Json<serde_json::Value>> {
    let pair = params.get("pair").cloned().unwrap_or_else(|| "SOL/USDC".to_string());
    let interval = params.get("interval").cloned().unwrap_or_else(|| "1m".to_string());

    // Parse pair
    let parts: Vec<&str> = pair.split('/').collect();
    if parts.len() != 2 {
        return Err(axum::response::Json(json!({
            "error": "Invalid pair format",
            "message": "Pair must be in format BASE/QUOTE"
        })));
    }

    let base_symbol = parts[0];
    let quote_symbol = parts[1];

    // Query ClickHouse for OHLCV data
    match state.clickhouse.get_ohlcv(base_symbol, quote_symbol, &interval).await {
        Ok(ohlcv_data) => Ok(Json(json!(ohlcv_data))),
        Err(e) => Err(axum::response::Json(json!({
            "error": "Failed to query OHLCV data",
            "message": format!("{}", e)
        })))
    }
}

pub fn routes() -> Router<std::sync::Arc<AppState>> {
    Router::new()
        .route("/trades", get(get_trades))
        .route("/ohlcv", get(get_ohlcv))
}
