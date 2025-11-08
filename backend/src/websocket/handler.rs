// WebSocket handler module

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use uuid::Uuid;

use crate::websocket::manager::ConnectionManager;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(manager): State<Arc<ConnectionManager>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, manager))
}

async fn handle_socket(socket: WebSocket, manager: Arc<ConnectionManager>) {
    let connection_id = Uuid::new_v4();
    println!("🔌 New WebSocket connection: {}", connection_id);

    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_rx = manager.add_connection(connection_id).await;

    // Channel for ping/pong handling
    let (ping_tx, mut ping_rx) = tokio::sync::mpsc::unbounded_channel();

    // Task to receive messages from client
    let manager_clone = manager.clone();
    let connection_id_clone = connection_id;
    let ping_tx_clone = ping_tx.clone();
    
    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(axum::extract::ws::Message::Text(text)) => {
                    println!("📥 Received from {}: {}", connection_id_clone, text);
                    // Handle client messages (e.g., pair selection)
                    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(msg_type) = msg.get("type").and_then(|v| v.as_str()) {
                            if msg_type == "select_pair" {
                                if let Some(pair) = msg.get("pair").and_then(|v| v.as_str()) {
                                    println!("📊 Pair selection received: {}", pair);
                                    let old_pair = manager_clone.get_selected_pair().await;
                                    manager_clone.set_selected_pair(pair.to_string()).await;
                                    let new_pair = manager_clone.get_selected_pair().await;
                                    println!("✅ Pair updated: {} -> {}", old_pair, new_pair);
                                } else {
                                    eprintln!("⚠️  Pair selection message missing 'pair' field");
                                }
                            } else {
                                println!("ℹ️  Received message type: {}", msg_type);
                            }
                        } else {
                            eprintln!("⚠️  Received message missing 'type' field");
                        }
                    } else {
                        eprintln!("⚠️  Failed to parse message as JSON: {}", text);
                    }
                }
                Ok(axum::extract::ws::Message::Close(_)) => {
                    println!("🔌 Connection closed: {}", connection_id_clone);
                    break;
                }
                Ok(axum::extract::ws::Message::Ping(data)) => {
                    // Send pong response via channel
                    let _ = ping_tx_clone.send(axum::extract::ws::Message::Pong(data));
                }
                Ok(axum::extract::ws::Message::Pong(_)) => {
                    // Pong received, no action needed
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
        manager_clone.remove_connection(connection_id_clone).await;
    });

    // Task to send messages to client (both broadcasts and pongs)
    let send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Handle broadcast messages
                result = broadcast_rx.recv() => {
                    match result {
                        Ok(msg) => {
                            // Log when messages are sent to client (only first few)
                            static SEND_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                            let send_count = SEND_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if send_count < 5 {
                                // Try to parse as trade to log it
                                if let Ok(trade) = serde_json::from_str::<serde_json::Value>(&msg) {
                                    if let (Some(id), Some(side), Some(amount), Some(price)) = (
                                        trade.get("id").and_then(|v| v.as_str()),
                                        trade.get("side").and_then(|v| v.as_str()),
                                        trade.get("amount").and_then(|v| v.as_f64()),
                                        trade.get("price").and_then(|v| v.as_f64()),
                                    ) {
                                        let base_symbol = trade.get("base_symbol").and_then(|v| v.as_str()).unwrap_or("?");
                                        let quote_symbol = trade.get("quote_symbol").and_then(|v| v.as_str()).unwrap_or("?");
                                        if side == "price" {
                                            println!("📤 [WS-SEND] Sending price update to client {}: {} {} @ ${:.6} (ID: {})", 
                                                connection_id, base_symbol, quote_symbol, price, &id[..16.min(id.len())]);
                                        } else {
                                            println!("📤 [WS-SEND] Sending trade to client {}: {} {:.6} {} @ ${:.6} (ID: {})", 
                                                connection_id, side, amount, base_symbol, price, &id[..16.min(id.len())]);
                                        }
                                    }
                                }
                            }
                            
                            if sender.send(axum::extract::ws::Message::Text(msg.into())).await.is_err() {
                                println!("❌ [WS-SEND] Failed to send message to client {}", connection_id);
                                break;
                            }
                        }
                        Err(_) => {
                            // Broadcast channel closed or lagged
                            println!("⚠️  [WS-SEND] Broadcast channel closed for client {}", connection_id);
                            break;
                        }
                    }
                }
                // Handle ping/pong
                Some(pong_msg) = ping_rx.recv() => {
                    if sender.send(pong_msg).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = receive_task => {
            println!("📥 Receive task completed for {}", connection_id);
        }
        _ = send_task => {
            println!("📤 Send task completed for {}", connection_id);
        }
    }

    manager.remove_connection(connection_id).await;
}

