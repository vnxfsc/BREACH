//! WebSocket handling for real-time updates

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};

use crate::AppState;

/// WebSocket query params
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    #[serde(default)]
    pub token: Option<String>,
    pub geohash: String,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    // Client -> Server
    #[serde(rename = "subscribe")]
    Subscribe { geohash: String },

    #[serde(rename = "unsubscribe")]
    Unsubscribe { geohash: String },

    #[serde(rename = "ping")]
    Ping,

    // Server -> Client
    #[serde(rename = "titan_spawn")]
    TitanSpawn {
        titan_id: String,
        location: Location,
        element: String,
        threat_class: i16,
        expires_at: String,
    },

    #[serde(rename = "titan_captured")]
    TitanCaptured {
        titan_id: String,
        captured_by: Option<String>,
    },

    #[serde(rename = "titan_expired")]
    TitanExpired { titan_id: String },

    #[serde(rename = "pong")]
    Pong,

    #[serde(rename = "subscribed")]
    Subscribed { geohash: String },

    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub lat: f64,
    pub lng: f64,
}

/// Global broadcast channels for geohash regions
pub struct Broadcaster {
    channels: RwLock<HashMap<String, broadcast::Sender<WsMessage>>>,
}

impl Broadcaster {
    pub fn new() -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
        }
    }

    /// Get or create a broadcast channel for a geohash prefix
    pub async fn get_channel(&self, geohash: &str) -> broadcast::Receiver<WsMessage> {
        let prefix = &geohash[..geohash.len().min(5)]; // Use 5-char prefix
        
        let mut channels = self.channels.write().await;
        
        if let Some(sender) = channels.get(prefix) {
            sender.subscribe()
        } else {
            let (tx, rx) = broadcast::channel(100);
            channels.insert(prefix.to_string(), tx);
            rx
        }
    }

    /// Broadcast a message to a geohash region
    pub async fn broadcast(&self, geohash: &str, message: WsMessage) {
        let prefix = &geohash[..geohash.len().min(5)];
        
        let channels = self.channels.read().await;
        
        if let Some(sender) = channels.get(prefix) {
            let _ = sender.send(message);
        }
    }
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new()
    }
}

/// WebSocket handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<WsQuery>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, query))
}

async fn handle_socket(socket: WebSocket, _state: Arc<AppState>, query: WsQuery) {
    let (mut sender, mut receiver) = socket.split();
    
    // Track subscribed geohashes
    let mut subscriptions: Vec<String> = vec![query.geohash.clone()];

    // Send initial subscription confirmation
    let confirm = WsMessage::Subscribed {
        geohash: query.geohash.clone(),
    };
    if let Ok(json) = serde_json::to_string(&confirm) {
        let _ = sender.send(Message::Text(json)).await;
    }

    // Main message loop
    while let Some(msg) = receiver.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => break,
        };

        match msg {
            Message::Text(text) => {
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    match ws_msg {
                        WsMessage::Subscribe { geohash } => {
                            if !subscriptions.contains(&geohash) {
                                subscriptions.push(geohash.clone());
                                let confirm = WsMessage::Subscribed { geohash };
                                if let Ok(json) = serde_json::to_string(&confirm) {
                                    let _ = sender.send(Message::Text(json)).await;
                                }
                            }
                        }
                        WsMessage::Unsubscribe { geohash } => {
                            subscriptions.retain(|g| g != &geohash);
                        }
                        WsMessage::Ping => {
                            if let Ok(json) = serde_json::to_string(&WsMessage::Pong) {
                                let _ = sender.send(Message::Text(json)).await;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Message::Ping(data) => {
                let _ = sender.send(Message::Pong(data)).await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    tracing::debug!("WebSocket connection closed");
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state)
}
