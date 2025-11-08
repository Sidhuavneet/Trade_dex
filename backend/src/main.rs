// Main application entry point

mod routes;
mod middleware;
mod websocket;
mod services;
mod models;
mod utils;
mod state;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use websocket::ConnectionManager;
use services::{TradeStreamService, ClickHouseService};
use state::AppState;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv().ok();
    
    // Initialize ClickHouse service
    let clickhouse = Arc::new(
        ClickHouseService::new().await
            .expect("Failed to initialize ClickHouse service")
    );
    
    // Initialize WebSocket connection manager
    let ws_manager = Arc::new(ConnectionManager::new());
    
    // Start trade stream service (fetches from QuickNode/Jupiter and broadcasts)
    let ws_manager_for_stream = ws_manager.clone();
    let clickhouse_for_stream = clickhouse.clone();
    tokio::spawn(async move {
        match TradeStreamService::new(ws_manager_for_stream, clickhouse_for_stream).await {
            Ok(stream_service) => {
                stream_service.start().await;
            }
            Err(e) => {
                eprintln!("❌ Failed to start trade stream service: {}", e);
                eprintln!("⚠️  Make sure QUICKNODE_RPC_URL and CLICKHOUSE_URL are set in environment variables");
            }
        }
    });

    // Shared state for routes
    let app_state = Arc::new(AppState {
        clickhouse: clickhouse.clone(),
    });

    let app = Router::new()
        .nest("/auth", routes::auth::routes().with_state(app_state.clone()))
        .nest("/api", routes::trades::routes().with_state(app_state.clone()))
        .route("/ws/trades", get(websocket::websocket_handler).with_state(ws_manager.clone()))
        .layer(middleware::create_cors_layer());

    // Bind to 0.0.0.0 to allow access from Docker containers
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));  
    println!("🚀 Server starting on http://{}", addr);
    println!("📡 WebSocket endpoint: ws://{}/ws/trades", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}