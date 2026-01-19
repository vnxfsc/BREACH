//! WebSocket handling for real-time updates

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
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::AppState;

/// WebSocket query params
#[derive(Debug, Deserialize)]
pub struct WsQuery {
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

    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub lat: f64,
    pub lng: f64,
}

/// WebSocket handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<WsQuery>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, query))
}

async fn handle_socket(mut socket: WebSocket, _state: Arc<AppState>, query: WsQuery) {
    // Track subscribed geohashes
    let mut subscriptions: Vec<String> = vec![query.geohash];

    // Main message loop
    while let Some(msg) = socket.recv().await {
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
                                subscriptions.push(geohash);
                            }
                        }
                        WsMessage::Unsubscribe { geohash } => {
                            subscriptions.retain(|g| g != &geohash);
                        }
                        WsMessage::Ping => {
                            let pong = serde_json::to_string(&WsMessage::Pong).unwrap();
                            if socket.send(Message::Text(pong)).await.is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state)
}
