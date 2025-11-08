// Trade model module

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub base_symbol: String,
    pub quote_symbol: String,
    pub base_mint: String,       // Token mint address for base token
    pub quote_mint: String,       // Token mint address for quote token
    pub price: f64,
    pub amount: f64,
    pub side: String,
    pub total_value: f64,        // price * amount
    pub dex_program: String,     // Jupiter v6, Jupiter v4, Raydium, Orca, Meteora, Phoenix
    pub slot: u64,               // Block slot number
}

