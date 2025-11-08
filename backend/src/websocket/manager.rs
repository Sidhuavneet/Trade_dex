// WebSocket connection manager module

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

pub type ConnectionId = Uuid;
pub type ConnectionMap = Arc<RwLock<HashMap<ConnectionId, broadcast::Sender<String>>>>;

#[derive(Clone)]
pub struct ConnectionManager {
    connections: ConnectionMap,
    broadcast_tx: broadcast::Sender<String>,
    selected_pair: Arc<RwLock<String>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
            selected_pair: Arc::new(RwLock::new("SOL/USDC".to_string())), // Default pair
        }
    }

    pub async fn add_connection(&self, id: ConnectionId) -> broadcast::Receiver<String> {
        let mut connections = self.connections.write().await;
        let receiver = self.broadcast_tx.subscribe();
        connections.insert(id, self.broadcast_tx.clone());
        println!("✅ WebSocket connection added: {}", id);
        receiver
    }

    pub async fn remove_connection(&self, id: ConnectionId) {
        let mut connections = self.connections.write().await;
        connections.remove(&id);
        println!("❌ WebSocket connection removed: {}", id);
    }

    pub async fn broadcast(&self, message: String) -> usize {
        let connections = self.connections.read().await;
        let count = connections.len();
        
        if count > 0 {
            match self.broadcast_tx.send(message) {
                Ok(_) => {
                    // Only log occasionally to reduce noise
                    static BROADCAST_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                    let bc_count = BROADCAST_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if bc_count < 5 || bc_count % 50 == 0 {
                        println!("📤 [WS-BROADCAST] Message sent to {} clients", count);
                    }
                }
                Err(e) => {
                    eprintln!("❌ [WS-BROADCAST] Failed to broadcast: {}", e);
                }
            }
        } else {
            // Log when no clients connected (but not too frequently)
            static NO_CLIENT_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let nc_count = NO_CLIENT_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if nc_count < 5 || nc_count % 100 == 0 {
                println!("⚠️  [WS-BROADCAST] No clients connected to receive message");
            }
        }
        
        count
    }

    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    pub async fn set_selected_pair(&self, pair: String) {
        let mut selected = self.selected_pair.write().await;
        let old_pair = selected.clone();
        *selected = pair.clone();
        println!("🔄 [ConnectionManager] Pair updated: {} -> {}", old_pair, pair);
    }

    pub async fn get_selected_pair(&self) -> String {
        self.selected_pair.read().await.clone()
    }
}

