// ClickHouse database service module
// Uses official clickhouse crate for ClickHouse Cloud

use crate::models::trade::Trade;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clickhouse::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct ClickHouseService {
    client: Arc<Client>,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
struct TradeRow {
    id: String,
    #[serde(with = "clickhouse::serde::time::datetime")]
    timestamp: OffsetDateTime,
    base_symbol: String,
    quote_symbol: String,
    price: f64,
    amount: f64,
    side: String,
}

#[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
struct SessionRow {
    user_pubkey: String,
    token: String,
    #[serde(with = "clickhouse::serde::time::datetime")]
    created_at: OffsetDateTime,
    #[serde(with = "clickhouse::serde::time::datetime")]
    expires_at: OffsetDateTime,
}

// Helper functions to convert between chrono::DateTime<Utc> and time::OffsetDateTime
fn chrono_to_time(dt: DateTime<Utc>) -> OffsetDateTime {
    let unix_timestamp = dt.timestamp();
    OffsetDateTime::from_unix_timestamp(unix_timestamp)
        .unwrap_or_else(|_| OffsetDateTime::now_utc())
}

fn time_to_chrono(dt: OffsetDateTime) -> DateTime<Utc> {
    let unix_timestamp = dt.unix_timestamp();
    DateTime::from_timestamp(unix_timestamp, 0)
        .unwrap_or_else(|| Utc::now())
}

impl ClickHouseService {
    pub async fn new() -> Result<Self> {
        // Get ClickHouse connection details from environment
        let clickhouse_url = std::env::var("CLICKHOUSE_URL")
            .unwrap_or_else(|_| "http://localhost:8123".to_string());
        
        let clickhouse_username = std::env::var("CLICKHOUSE_USERNAME")
            .unwrap_or_else(|_| "default".to_string());
        
        let clickhouse_password = std::env::var("CLICKHOUSE_PASSWORD")
            .unwrap_or_else(|_| "".to_string());
        
        // Create ClickHouse client using official crate
        // URL should include protocol and port: https://instance.clickhouse.cloud:8443
        let client = Client::default()
            .with_url(&clickhouse_url)
            .with_user(&clickhouse_username)
            .with_password(&clickhouse_password)
            .with_database("default");
        
        let service = Self {
            client: Arc::new(client),
        };
        
        // Test connection
        match service.test_connection().await {
            Ok(_) => {
                println!("✅ ClickHouse connection successful");
            }
            Err(e) => {
                eprintln!("⚠️  ClickHouse connection test failed: {}", e);
                eprintln!("   URL: {}", clickhouse_url);
                eprintln!("   Username: {}", clickhouse_username);
                eprintln!("   This may cause insert failures. Please check your CLICKHOUSE_URL, CLICKHOUSE_USERNAME, and CLICKHOUSE_PASSWORD environment variables.");
            }
        }
        
        // Initialize tables
        service.init_tables().await?;
        
        Ok(service)
    }
    
    /// Test ClickHouse connection
    async fn test_connection(&self) -> Result<()> {
        self.client
            .query("SELECT 1")
            .fetch_one::<u8>()
            .await
            .context("Failed to test ClickHouse connection")?;
        Ok(())
    }
    
    /// Initialize ClickHouse tables
    async fn init_tables(&self) -> Result<()> {
        // Create trades table - matching assignment schema
        let trades_sql = "CREATE TABLE IF NOT EXISTS trades (
            id String,
            timestamp DateTime,
            base_symbol String,
            quote_symbol String,
            price Float64,
            amount Float64,
            side String
        ) ENGINE = MergeTree()
        ORDER BY timestamp";
        
        self.client
            .query(trades_sql)
            .execute()
            .await
            .context("Failed to create trades table")?;
        
        println!("✅ ClickHouse trades table initialized");
        
        // Create sessions table for user sessions
        // Using DateTime('UTC') to ensure timezone consistency
        let sessions_sql = "CREATE TABLE IF NOT EXISTS sessions (
            user_pubkey String,
            token String,
            created_at DateTime('UTC'),
            expires_at DateTime('UTC')
        ) ENGINE = MergeTree()
        ORDER BY (user_pubkey, expires_at)";
        
        self.client
            .query(sessions_sql)
            .execute()
            .await
            .context("Failed to create sessions table")?;
        
        println!("✅ ClickHouse sessions table initialized");
        
        Ok(())
    }
    
    /// Store a trade in ClickHouse
    /// Uses the inserter pattern for type-safe insertion (recommended by ClickHouse Rust client docs)
    pub async fn store_trade(&self, trade: &Trade) -> Result<()> {
        // Create TradeRow for insertion - convert chrono::DateTime<Utc> to time::OffsetDateTime
        // Only store fields required by assignment schema
        let trade_row = TradeRow {
            id: trade.id.clone(),
            timestamp: chrono_to_time(trade.timestamp),
            base_symbol: trade.base_symbol.clone(),
            quote_symbol: trade.quote_symbol.clone(),
            price: trade.price,
            amount: trade.amount,
            side: trade.side.clone(),
        };
        
        println!("📝 Attempting to insert trade: {} {} {} @ ${:.6}", trade.side, trade.amount, trade.base_symbol, trade.price);
        
        // Use inserter pattern (type-safe, recommended by ClickHouse Rust client docs)
        let mut inserter = self.client
            .inserter("trades")?
            .with_max_rows(1);
        
        inserter.write(&trade_row)?; // write() is not async, remove .await
        inserter.end().await?;
        
        println!("✅ Successfully inserted trade: {} {} {} @ ${:.6}", trade.side, trade.amount, trade.base_symbol, trade.price);
        
        Ok(())
    }
    
    /// Store a user session in ClickHouse
    /// Uses the inserter pattern for type-safe insertion
    pub async fn store_session(
        &self,
        user_pubkey: &str,
        token: &str,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<()> {
        let created_at = Utc::now();
        
        println!("📝 Attempting to store session in ClickHouse: user={}, created_at={}, expires_at={}", 
            user_pubkey, created_at, expires_at);
        
        // Create SessionRow for insertion - convert chrono::DateTime<Utc> to time::OffsetDateTime
        let session_row = SessionRow {
            user_pubkey: user_pubkey.to_string(),
            token: token.to_string(),
            created_at: chrono_to_time(created_at),
            expires_at: chrono_to_time(expires_at),
        };
        
        // Use inserter pattern (type-safe, recommended by ClickHouse Rust client docs)
        let mut inserter = self.client
            .inserter("sessions")?
            .with_max_rows(1);
        
        inserter.write(&session_row)?; // write() is not async, remove .await
        inserter.end().await?;
        
        println!("✅ Session stored in ClickHouse for user: {}", user_pubkey);
        
        Ok(())
    }
    
    /// Get recent trades filtered by pair
    pub async fn get_trades(
        &self,
        base_symbol: &str,
        quote_symbol: &str,
        limit: usize,
    ) -> Result<Vec<Trade>> {
        
        // Query - DateTime<Utc> is handled automatically by serde with time feature
        // Must select columns in the exact order of TradeRow struct
        // Filter by pair in both directions (SOL/USDC or USDC/SOL)
        let query_result = self.client
            .query("SELECT id, timestamp, base_symbol, quote_symbol, price, amount, side
                    FROM trades
                    WHERE (base_symbol = ? AND quote_symbol = ?) OR (base_symbol = ? AND quote_symbol = ?)
                    ORDER BY timestamp DESC
                    LIMIT ?")
            .bind(base_symbol)
            .bind(quote_symbol)
            .bind(quote_symbol)  // Reverse direction
            .bind(base_symbol)    // Reverse direction
            .bind(limit as u64)
            .fetch_all::<TradeRow>()
            .await;
        
        let cursor = match query_result {
            Ok(cursor) => {
                println!("✅ ClickHouse query successful, fetched {} rows", cursor.len());
                cursor
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                eprintln!("❌ ClickHouse query error: {}", error_msg);
                eprintln!("   Query: SELECT ... FROM trades WHERE (base_symbol = '{}' AND quote_symbol = '{}') OR (base_symbol = '{}' AND quote_symbol = '{}') LIMIT {}", base_symbol, quote_symbol, quote_symbol, base_symbol, limit);
                return Err(e).with_context(|| format!("Failed to query trades from ClickHouse for pair {}/{}: {}", base_symbol, quote_symbol, error_msg));
            }
        };
        
        // Convert to Trade structs - convert time::OffsetDateTime back to chrono::DateTime<Utc>
        // Note: Extra fields (base_mint, quote_mint, total_value, dex_program, slot) are not stored in ClickHouse
        // They will be set to default values when reading from ClickHouse
        let trades: Vec<Trade> = cursor
            .iter()
            .map(|row| Trade {
                id: row.id.clone(),
                timestamp: time_to_chrono(row.timestamp),
                base_symbol: row.base_symbol.clone(),
                quote_symbol: row.quote_symbol.clone(),
                base_mint: String::new(), // Not stored in ClickHouse per assignment
                quote_mint: String::new(), // Not stored in ClickHouse per assignment
                price: row.price,
                amount: row.amount,
                side: row.side.clone(),
                total_value: row.price * row.amount, // Calculate from stored price and amount
                dex_program: String::new(), // Not stored in ClickHouse per assignment
                slot: 0, // Not stored in ClickHouse per assignment
            })
            .collect();
        
        Ok(trades)
    }
    
    /// Get OHLCV data aggregated from ClickHouse
    pub async fn get_ohlcv(
        &self,
        base_symbol: &str,
        quote_symbol: &str,
        interval: &str,
    ) -> Result<Vec<serde_json::Value>> {
        // Define row struct for OHLCV aggregation results
        #[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
        struct OHLCVRow {
            time: u64,
            open: f64,
            high: f64,
            low: f64,
            close: f64,
            volume: f64,
        }
        
        // Convert interval to ClickHouse format
        let interval_sql = match interval {
            "1m" => "1 MINUTE",
            "5m" => "5 MINUTE",
            "15m" => "15 MINUTE",
            "1h" => "1 HOUR",
            "4h" => "4 HOUR",
            "1d" => "1 DAY",
            _ => "1 MINUTE",
        };
        
        // Query with OHLC aggregation
        let cursor = self.client
            .query(&format!(
                "SELECT
                    toUnixTimestamp(toStartOfInterval(timestamp, INTERVAL {})) as time,
                    argMin(price, timestamp) as open,
                    max(price) as high,
                    min(price) as low,
                    argMax(price, timestamp) as close,
                    sum(amount * price) as volume
                FROM trades
                WHERE base_symbol = ? AND quote_symbol = ?
                GROUP BY time
                ORDER BY time ASC",
                interval_sql
            ))
            .bind(base_symbol)
            .bind(quote_symbol)
            .fetch_all::<OHLCVRow>()
            .await
            .context("Failed to query OHLCV from ClickHouse")?;
        
        // Convert to JSON format
        let ohlcv_data: Vec<serde_json::Value> = cursor
            .iter()
            .map(|row| {
                json!({
                    "time": row.time,
                    "open": row.open,
                    "high": row.high,
                    "low": row.low,
                    "close": row.close,
                    "volume": row.volume,
                })
            })
            .collect();
        
        Ok(ohlcv_data)
    }
    
    /// Get 24h stats for a pair
    pub async fn get_24h_stats(
        &self,
        base_symbol: &str,
        quote_symbol: &str,
    ) -> Result<serde_json::Value> {
        // Define row struct for 24h stats
        #[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
        struct StatsRow {
            current_price: f64,
            low_24h: f64,
            high_24h: f64,
            volume_24h: f64,
            first_price: f64,
            last_price: f64,
        }
        
        let cursor = self.client
            .query("SELECT
                argMax(price, timestamp) as current_price,
                min(price) as low_24h,
                max(price) as high_24h,
                sum(amount * price) as volume_24h,
                argMin(price, timestamp) as first_price,
                argMax(price, timestamp) as last_price
            FROM trades
            WHERE base_symbol = ? AND quote_symbol = ?
            AND timestamp >= now() - INTERVAL 24 HOUR")
            .bind(base_symbol)
            .bind(quote_symbol)
            .fetch_all::<StatsRow>()
            .await
            .context("Failed to query 24h stats from ClickHouse")?;
        
        if let Some(row) = cursor.first() {
            let change_24h = row.last_price - row.first_price;
            let change_percent_24h = if row.first_price > 0.0 {
                (change_24h / row.first_price) * 100.0
            } else {
                0.0
            };
            
            return Ok(json!({
                "currentPrice": row.current_price,
                "high24h": row.high_24h,
                "low24h": row.low_24h,
                "volume24h": row.volume_24h,
                "change24h": change_24h,
                "changePercent24h": change_percent_24h,
            }));
        }
        
        Ok(json!({
            "currentPrice": 0.0,
            "high24h": 0.0,
            "low24h": 0.0,
            "volume24h": 0.0,
            "change24h": 0.0,
            "changePercent24h": 0.0,
        }))
    }
    
    /// Check if a session is valid
    pub async fn validate_session(&self, user_pubkey: &str, token: &str) -> Result<bool> {
        #[derive(Debug, Serialize, Deserialize, clickhouse::Row)]
        struct SessionCheck {
            count: u8,
        }
        
        let cursor = self.client
            .query("SELECT 1 as count
                    FROM sessions
                    WHERE user_pubkey = ? AND token = ? AND expires_at > now()
                    LIMIT 1")
            .bind(user_pubkey)
            .bind(token)
            .fetch_all::<SessionCheck>()
            .await
            .context("Failed to validate session in ClickHouse")?;
        
        Ok(!cursor.is_empty())
    }
    
    /// Delete expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<()> {
        self.client
            .query("ALTER TABLE sessions DELETE WHERE expires_at < now()")
            .execute()
            .await
            .context("Failed to cleanup expired sessions")?;
        
        Ok(())
    }
}
