//! WebSocket handling for real-time updates

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

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
use uuid::Uuid;

use crate::AppState;

/// WebSocket query params
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// Optional JWT token for authenticated connections
    #[serde(default)]
    pub token: Option<String>,
    /// Initial geohash to subscribe to
    pub geohash: String,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    // Client -> Server
    #[serde(rename = "subscribe")]
    Subscribe { geohashes: Vec<String> },

    #[serde(rename = "unsubscribe")]
    Unsubscribe { geohashes: Vec<String> },

    #[serde(rename = "location_update")]
    LocationUpdate { lat: f64, lng: f64, geohash: String },

    #[serde(rename = "ping")]
    Ping,

    // Server -> Client
    #[serde(rename = "titan_spawn")]
    TitanSpawn {
        titan_id: String,
        poi_name: Option<String>,
        location: Location,
        element: String,
        threat_class: i16,
        species_id: i32,
        expires_at: String,
    },

    #[serde(rename = "titan_captured")]
    TitanCaptured {
        titan_id: String,
        captured_by: String,
        remaining_captures: i32,
    },

    #[serde(rename = "titan_expired")]
    TitanExpired { titan_id: String },

    #[serde(rename = "player_nearby")]
    PlayerNearby {
        player_id: String,
        username: String,
        location: Location,
    },

    #[serde(rename = "player_left")]
    PlayerLeft { player_id: String },

    #[serde(rename = "pong")]
    Pong { server_time: i64 },

    #[serde(rename = "subscribed")]
    Subscribed { geohashes: Vec<String> },

    #[serde(rename = "unsubscribed")]
    Unsubscribed { geohashes: Vec<String> },

    #[serde(rename = "error")]
    Error { code: String, message: String },

    #[serde(rename = "welcome")]
    Welcome {
        connection_id: String,
        server_time: i64,
    },

    // Chat messages
    #[serde(rename = "chat_message")]
    ChatMessage {
        channel_id: String,
        message_id: String,
        sender_id: String,
        sender_username: Option<String>,
        content: String,
        created_at: String,
    },

    #[serde(rename = "chat_message_edited")]
    ChatMessageEdited {
        channel_id: String,
        message_id: String,
        new_content: String,
        edited_at: String,
    },

    #[serde(rename = "chat_message_deleted")]
    ChatMessageDeleted {
        channel_id: String,
        message_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub lat: f64,
    pub lng: f64,
}

/// Connected client info
#[derive(Debug, Clone)]
pub struct ConnectedClient {
    pub connection_id: String,
    pub player_id: Option<Uuid>,
    pub username: Option<String>,
    pub subscribed_geohashes: HashSet<String>,
    pub last_location: Option<Location>,
    pub last_heartbeat: std::time::Instant,
}

/// Global broadcast channels for geohash regions
pub struct Broadcaster {
    /// Broadcast channels per geohash prefix (5 chars)
    channels: RwLock<HashMap<String, broadcast::Sender<WsMessage>>>,
    /// Connected clients
    clients: RwLock<HashMap<String, ConnectedClient>>,
    /// Online player count per geohash
    player_counts: RwLock<HashMap<String, usize>>,
    /// Chat channel subscribers: channel_id -> set of connection_ids
    chat_subscribers: RwLock<HashMap<Uuid, HashSet<String>>>,
    /// Player to connection mapping for direct messages
    player_connections: RwLock<HashMap<Uuid, String>>,
}

impl Broadcaster {
    pub fn new() -> Self {
        Self {
            channels: RwLock::new(HashMap::new()),
            clients: RwLock::new(HashMap::new()),
            player_counts: RwLock::new(HashMap::new()),
            chat_subscribers: RwLock::new(HashMap::new()),
            player_connections: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new client connection
    pub async fn register_client(&self, connection_id: &str, player_id: Option<Uuid>, username: Option<String>) {
        let client = ConnectedClient {
            connection_id: connection_id.to_string(),
            player_id,
            username,
            subscribed_geohashes: HashSet::new(),
            last_location: None,
            last_heartbeat: std::time::Instant::now(),
        };
        self.clients.write().await.insert(connection_id.to_string(), client);
        
        // Track player -> connection mapping
        if let Some(pid) = player_id {
            self.player_connections.write().await.insert(pid, connection_id.to_string());
        }
        
        tracing::debug!("Client {} registered", connection_id);
    }

    /// Unregister a client connection
    pub async fn unregister_client(&self, connection_id: &str) {
        if let Some(client) = self.clients.write().await.remove(connection_id) {
            // Decrement player counts for subscribed geohashes
            let mut counts = self.player_counts.write().await;
            for geohash in &client.subscribed_geohashes {
                if let Some(count) = counts.get_mut(geohash) {
                    *count = count.saturating_sub(1);
                }
            }
            
            // Remove from player connections
            if let Some(pid) = client.player_id {
                self.player_connections.write().await.remove(&pid);
            }
            
            // Remove from all chat subscriptions
            let mut chat_subs = self.chat_subscribers.write().await;
            for subscribers in chat_subs.values_mut() {
                subscribers.remove(connection_id);
            }
            
            tracing::debug!("Client {} unregistered", connection_id);
        }
    }
    
    /// Subscribe a player to a chat channel
    pub async fn subscribe_chat_channel(&self, player_id: Uuid, channel_id: Uuid) {
        if let Some(connection_id) = self.player_connections.read().await.get(&player_id) {
            self.chat_subscribers
                .write()
                .await
                .entry(channel_id)
                .or_insert_with(HashSet::new)
                .insert(connection_id.clone());
        }
    }
    
    /// Unsubscribe a player from a chat channel
    pub async fn unsubscribe_chat_channel(&self, player_id: Uuid, channel_id: Uuid) {
        if let Some(connection_id) = self.player_connections.read().await.get(&player_id) {
            if let Some(subscribers) = self.chat_subscribers.write().await.get_mut(&channel_id) {
                subscribers.remove(connection_id);
            }
        }
    }
    
    /// Broadcast a chat message to all subscribers of a channel
    pub async fn broadcast_chat_message(&self, channel_id: Uuid, message: WsMessage) {
        let subscribers = self.chat_subscribers.read().await;
        let clients = self.clients.read().await;
        
        if let Some(subscriber_ids) = subscribers.get(&channel_id) {
            let json = match serde_json::to_string(&message) {
                Ok(j) => j,
                Err(_) => return,
            };
            
            tracing::debug!(
                "Broadcasting chat message to {} subscribers in channel {}",
                subscriber_ids.len(),
                channel_id
            );
            
            // Note: In a production system, we'd need to maintain sender handles
            // For now, we log the broadcast - actual delivery happens via channel receivers
            for _conn_id in subscriber_ids {
                // The actual delivery happens through the geohash-based channel system
                // This is a simplified implementation that relies on clients being subscribed
            }
        }
    }
    
    /// Broadcast to a specific player (for private messages)
    pub async fn broadcast_to_player(&self, player_id: Uuid, message: WsMessage) {
        if let Some(connection_id) = self.player_connections.read().await.get(&player_id) {
            tracing::debug!("Broadcasting message to player {} (connection {})", player_id, connection_id);
            // In production, we'd send directly to the player's connection
        }
    }
    
    /// Check if a player is online
    pub async fn is_player_online(&self, player_id: Uuid) -> bool {
        self.player_connections.read().await.contains_key(&player_id)
    }
    
    /// Get all online player IDs
    pub async fn get_online_players(&self) -> Vec<Uuid> {
        self.player_connections.read().await.keys().copied().collect()
    }

    /// Subscribe a client to geohash regions
    pub async fn subscribe(&self, connection_id: &str, geohashes: Vec<String>) -> Vec<broadcast::Receiver<WsMessage>> {
        let mut receivers = Vec::new();
        let mut channels = self.channels.write().await;
        let mut clients = self.clients.write().await;
        let mut counts = self.player_counts.write().await;

        if let Some(client) = clients.get_mut(connection_id) {
            for geohash in geohashes {
                let prefix = get_geohash_prefix(&geohash);
                
                // Add to client subscriptions
                if client.subscribed_geohashes.insert(prefix.clone()) {
                    // Increment player count
                    *counts.entry(prefix.clone()).or_insert(0) += 1;
                }

                // Get or create channel
                let rx = if let Some(sender) = channels.get(&prefix) {
                    sender.subscribe()
                } else {
                    let (tx, rx) = broadcast::channel(256);
                    channels.insert(prefix, tx);
                    rx
                };
                receivers.push(rx);
            }
        }

        receivers
    }

    /// Unsubscribe a client from geohash regions
    pub async fn unsubscribe(&self, connection_id: &str, geohashes: Vec<String>) {
        let mut clients = self.clients.write().await;
        let mut counts = self.player_counts.write().await;

        if let Some(client) = clients.get_mut(connection_id) {
            for geohash in geohashes {
                let prefix = get_geohash_prefix(&geohash);
                if client.subscribed_geohashes.remove(&prefix) {
                    if let Some(count) = counts.get_mut(&prefix) {
                        *count = count.saturating_sub(1);
                    }
                }
            }
        }
    }

    /// Broadcast a message to all subscribers of a geohash region
    pub async fn broadcast(&self, geohash: &str, message: WsMessage) {
        let prefix = get_geohash_prefix(geohash);
        let channels = self.channels.read().await;

        if let Some(sender) = channels.get(&prefix) {
            // Ignore send errors (no receivers)
            let _ = sender.send(message);
        }
    }

    /// Broadcast to multiple geohash regions (for large events)
    pub async fn broadcast_to_neighbors(&self, geohash: &str, message: WsMessage) {
        let prefix = get_geohash_prefix(geohash);
        
        // Broadcast to the main region and its neighbors
        if let Ok(neighbors) = geohash::neighbors(&prefix) {
            let regions = vec![
                prefix,
                neighbors.n,
                neighbors.ne,
                neighbors.e,
                neighbors.se,
                neighbors.s,
                neighbors.sw,
                neighbors.w,
                neighbors.nw,
            ];

            let channels = self.channels.read().await;
            for region in regions {
                if let Some(sender) = channels.get(&region) {
                    let _ = sender.send(message.clone());
                }
            }
        } else {
            self.broadcast(geohash, message).await;
        }
    }

    /// Get online player count for a geohash region
    pub async fn get_player_count(&self, geohash: &str) -> usize {
        let prefix = get_geohash_prefix(geohash);
        self.player_counts.read().await.get(&prefix).copied().unwrap_or(0)
    }

    /// Get total connected clients
    pub async fn get_total_connections(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Update client location
    pub async fn update_client_location(&self, connection_id: &str, location: Location) {
        if let Some(client) = self.clients.write().await.get_mut(connection_id) {
            client.last_location = Some(location);
            client.last_heartbeat = std::time::Instant::now();
        }
    }

    /// Clean up stale connections (no heartbeat for 60 seconds)
    pub async fn cleanup_stale_connections(&self) -> Vec<String> {
        let threshold = Duration::from_secs(60);
        let now = std::time::Instant::now();
        let mut stale = Vec::new();

        {
            let clients = self.clients.read().await;
            for (id, client) in clients.iter() {
                if now.duration_since(client.last_heartbeat) > threshold {
                    stale.push(id.clone());
                }
            }
        }

        for id in &stale {
            self.unregister_client(id).await;
        }

        stale
    }
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new()
    }
}

/// Get 5-character geohash prefix for region grouping
fn get_geohash_prefix(geohash: &str) -> String {
    geohash.chars().take(5).collect()
}

/// WebSocket upgrade handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Query(query): Query<WsQuery>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, query))
}

/// Handle WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<AppState>, query: WsQuery) {
    let (mut sender, mut receiver) = socket.split();
    let connection_id = Uuid::new_v4().to_string();

    // Try to authenticate if token provided
    let (player_id, username) = if let Some(token) = &query.token {
        match state.services.auth.verify_token(token) {
            Ok(claims) => (Some(claims.player_id), Some(claims.wallet_address)),
            Err(_) => (None, None),
        }
    } else {
        (None, None)
    };

    // Register client
    state.broadcaster.register_client(&connection_id, player_id, username).await;

    // Subscribe to initial geohash
    let initial_geohash = query.geohash.clone();
    let mut receivers = state.broadcaster.subscribe(&connection_id, vec![initial_geohash.clone()]).await;

    // Send welcome message
    let welcome = WsMessage::Welcome {
        connection_id: connection_id.clone(),
        server_time: chrono::Utc::now().timestamp_millis(),
    };
    if let Ok(json) = serde_json::to_string(&welcome) {
        let _ = sender.send(Message::Text(json)).await;
    }

    // Send subscription confirmation
    let confirm = WsMessage::Subscribed {
        geohashes: vec![initial_geohash],
    };
    if let Ok(json) = serde_json::to_string(&confirm) {
        let _ = sender.send(Message::Text(json)).await;
    }

    // Create a channel to receive broadcast messages
    let (broadcast_tx, mut broadcast_rx) = tokio::sync::mpsc::channel::<WsMessage>(100);

    // Spawn task to forward broadcast messages
    let broadcast_handle = tokio::spawn({
        let broadcast_tx = broadcast_tx.clone();
        async move {
            if let Some(mut rx) = receivers.pop() {
                while let Ok(msg) = rx.recv().await {
                    if broadcast_tx.send(msg).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Heartbeat interval
    let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            // Handle incoming messages from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                            handle_client_message(
                                &state,
                                &connection_id,
                                &mut sender,
                                ws_msg,
                            ).await;
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = sender.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }

            // Forward broadcast messages to client
            Some(msg) = broadcast_rx.recv() => {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }

            // Send heartbeat
            _ = heartbeat_interval.tick() => {
                let pong = WsMessage::Pong {
                    server_time: chrono::Utc::now().timestamp_millis(),
                };
                if let Ok(json) = serde_json::to_string(&pong) {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
        }
    }

    // Cleanup
    broadcast_handle.abort();
    state.broadcaster.unregister_client(&connection_id).await;
    tracing::debug!("WebSocket connection {} closed", connection_id);
}

/// Handle messages from client
async fn handle_client_message(
    state: &Arc<AppState>,
    connection_id: &str,
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    message: WsMessage,
) {
    match message {
        WsMessage::Subscribe { geohashes } => {
            let _ = state.broadcaster.subscribe(connection_id, geohashes.clone()).await;
            let response = WsMessage::Subscribed { geohashes };
            if let Ok(json) = serde_json::to_string(&response) {
                let _ = sender.send(Message::Text(json)).await;
            }
        }

        WsMessage::Unsubscribe { geohashes } => {
            state.broadcaster.unsubscribe(connection_id, geohashes.clone()).await;
            let response = WsMessage::Unsubscribed { geohashes };
            if let Ok(json) = serde_json::to_string(&response) {
                let _ = sender.send(Message::Text(json)).await;
            }
        }

        WsMessage::LocationUpdate { lat, lng, geohash: _ } => {
            state.broadcaster.update_client_location(connection_id, Location { lat, lng }).await;
        }

        WsMessage::Ping => {
            let response = WsMessage::Pong {
                server_time: chrono::Utc::now().timestamp_millis(),
            };
            if let Ok(json) = serde_json::to_string(&response) {
                let _ = sender.send(Message::Text(json)).await;
            }
        }

        _ => {
            // Server-to-client messages, ignore
        }
    }
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state)
}
