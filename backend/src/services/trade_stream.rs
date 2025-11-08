// Trade stream processing service module

use crate::models::trade::Trade;
use crate::services::jupiter::JupiterService;
use crate::services::solana::SolanaService;
use crate::services::quicknode_ws::QuickNodeWebSocket;
use crate::services::clickhouse::ClickHouseService;
use crate::services::pair_mapping::{pair_to_mints, parse_pair};
use crate::websocket::ConnectionManager;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

pub struct TradeStreamService {
    solana: SolanaService,
    jupiter: JupiterService,
    clickhouse: Arc<ClickHouseService>,
    ws_manager: Arc<ConnectionManager>,
}

impl TradeStreamService {
    pub async fn new(
        ws_manager: Arc<ConnectionManager>,
        clickhouse: Arc<ClickHouseService>,
    ) -> Result<Self> {
        let solana = SolanaService::new()?;
        
        // Cleanup expired sessions periodically
        let clickhouse_clone = clickhouse.clone();
        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(3600)); // Every hour
            loop {
                cleanup_interval.tick().await;
                if let Err(e) = clickhouse_clone.cleanup_expired_sessions().await {
                    eprintln!("⚠️  Failed to cleanup expired sessions: {}", e);
                }
            }
        });
        
        Ok(Self {
            solana,
            jupiter: JupiterService::new()?,
            clickhouse,
            ws_manager,
        })
    }

    /// Start the trade stream service
    pub async fn start(&self) {
        println!("🚀 Starting trade stream service...");
        
        let solana_service = Arc::new(self.solana.clone());
        let ws_manager = self.ws_manager.clone();
        let jupiter = self.jupiter.clone();
        let clickhouse = self.clickhouse.clone();
        
        // Channel for QuickNode WebSocket trades
        let (trade_tx, mut trade_rx) = mpsc::channel::<Trade>(100);
        
        // Start QuickNode WebSocket subscription
        let quicknode_ws = QuickNodeWebSocket::new(solana_service.clone())
            .expect("Failed to create QuickNode WebSocket client");
        
        let quicknode_ws_clone = quicknode_ws.clone();
        let trade_tx_clone = trade_tx.clone();
        
        // Spawn QuickNode WebSocket subscription task
        tokio::spawn(async move {
            loop {
                match quicknode_ws_clone.start_subscription(trade_tx_clone.clone()).await {
                    Ok(_) => {
                        eprintln!("⚠️  QuickNode WebSocket closed, reconnecting...");
                    }
                    Err(e) => {
                        eprintln!("❌ QuickNode WebSocket error: {}", e);
                    }
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
        
        // Spawn Jupiter price update task (every 5 seconds)
        let jupiter_clone = jupiter.clone();
        let ws_manager_price = ws_manager.clone();
        tokio::spawn(async move {
            let mut price_interval = interval(Duration::from_secs(5));
            let mut tick_count = 0u64;
            loop {
                price_interval.tick().await;
                tick_count += 1;
                
                // Get current selected pair
                let selected_pair = ws_manager_price.get_selected_pair().await;
                println!("🔄 [PriceUpdate] Current selected pair: {}", selected_pair);
                
                // Parse pair and get mint addresses
                if let Some((base_mint, quote_mint)) = pair_to_mints(&selected_pair) {
                    if let Some((base_symbol, quote_symbol)) = parse_pair(&selected_pair) {
                        match jupiter_clone.get_price(&base_mint, &quote_mint).await {
                            Ok(price) => {
                                println!("💰 Jupiter price fetched: {} {} @ ${:.6}", base_symbol, quote_symbol, price);
                                let price_trade = serde_json::json!({
                                    "id": format!("price_{}", Utc::now().timestamp()),
                                    "timestamp": Utc::now().to_rfc3339(),
                                    "base_symbol": base_symbol,
                                    "quote_symbol": quote_symbol,
                                    "price": price,
                                    "amount": 0.0,
                                    "side": "price"
                                });
                                
                                if let Ok(price_json) = serde_json::to_string(&price_trade) {
                                    let client_count = ws_manager_price.broadcast(price_json).await;
                                    println!("📤 [PRICE-UPDATE] Broadcasting {} {} @ ${:.6} to {} clients", 
                                        base_symbol, quote_symbol, price, client_count);
                                } else {
                                    eprintln!("❌ Failed to serialize price update JSON");
                                }
                            }
                            Err(e) => {
                                eprintln!("⚠️  Failed to fetch Jupiter price for {}: {}", selected_pair, e);
                            }
                        }
                    }
                } else {
                    eprintln!("⚠️  Invalid pair format: {}", selected_pair);
                }
            }
        });
        
        // Process trades from QuickNode WebSocket
        loop {
            tokio::select! {
                // Receive trades from QuickNode WebSocket
                Some(trade) = trade_rx.recv() => {
                    // Get current price from Jupiter for validation
                    let current_price = match jupiter.get_sol_usdc_price().await {
                        Ok(price) => price,
                        Err(_) => 150.0, // Fallback
                    };
                    
                    let mut trade = trade;
                    
                    // Validate price
                    if trade.price <= 0.0 || trade.price.is_infinite() || trade.price.is_nan() {
                        trade.price = current_price;
                    }
                    
                    // Store trade in ClickHouse
                    if let Err(e) = clickhouse.store_trade(&trade).await {
                        eprintln!("❌ Failed to store trade in ClickHouse: {}", e);
                        eprintln!("   Trade details: {} {} {} @ ${:.6} (ID: {})", 
                            trade.side, trade.amount, trade.base_symbol, trade.price, 
                            &trade.id[..16.min(trade.id.len())]);
                    } else {
                        println!("✅ Stored trade in ClickHouse: {} {} {} @ ${:.6}", 
                            trade.side, trade.amount, trade.base_symbol, trade.price);
                    }
                    
                    // Broadcast via WebSocket
                    if let Ok(trade_json) = serde_json::to_string(&trade) {
                        let client_count = ws_manager.broadcast(trade_json.clone()).await;
                        println!("send_trade: {} {:.6} SOL @ ${:.2} to {} clients", 
                            trade.side, trade.amount, trade.price, client_count);
                    }
                }
            }
        }
    }

}
