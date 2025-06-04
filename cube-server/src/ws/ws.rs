//! # WebSocket Handler
//!
//! This module provides WebSocket support for real-time communication between the server and clients.
//!
//! ## Features
//! - Accepts WebSocket connections and manages a list of connected clients.
//! - Allows broadcasting JSON messages to all connected clients.
//! - Handles incoming messages and dispatches actions based on their content (e.g., "copy_files").
//!
//! ## Main Functions
//! - `ws_handler`: Axum handler to upgrade HTTP requests to WebSocket connections.
//! - `handle_socket`: Manages the lifecycle of a WebSocket connection, including receiving and sending messages.
//! - `broadcast_json`: Broadcasts a JSON message to all connected clients.

use crate::state::AppState;
use axum::extract::ws::{Message, WebSocketUpgrade, WebSocket};
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::{Mutex, mpsc::UnboundedSender};
use serde_json::Value;

/// Type alias for the list of connected WebSocket clients.
pub type Clients = Arc<Mutex<Vec<UnboundedSender<Message>>>>;

/// Axum handler to upgrade HTTP requests to WebSocket connections.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handles the lifecycle of a WebSocket connection.
///
/// - Adds the client to the global client list.
/// - Spawns a task to send messages to the client.
/// - Listens for incoming messages and dispatches actions.
/// - Removes the client on disconnect.
async fn handle_socket(stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    // Add this client to the list
    state.ws_state.lock().await.push(tx.clone());

    // Task to send messages to this socket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let _ = sender.send(msg).await;
        }
    });

    // Listen for incoming messages from this socket
    while let Some(Ok(Message::Text(text))) = receiver.next().await {
        println!("📩 WS received: {}", text);

        match serde_json::from_str::<Value>(&text) {
            Ok(payload) => {
                if let Some(action) = payload.get("name").and_then(|a| a.as_str()) {
                    match action {
                        "copy_files" => {
                            if let Some(hashes) = payload.get("payload").and_then(|h| h.get("hashes")).and_then(|h| h.as_array()) {
                                let clients = state.ws_state.lock().await;

                                for hash in hashes {
                                    if let Some(h) = hash.as_str() {
                                        let request = serde_json::json!({
                                            "action": "send_raw",
                                            "hash": h
                                        });

                                        println!("⬇️ Sending download for {}", h);
                                        broadcast_json(&clients, &request);
                                    }
                                }
                            }
                        }

                        _ => println!("⚠️ Unknown action: {}", action),
                    }
                }

            }
            Err(err) => {
                println!("❌ Error decoding WS JSON: {err}");
            }
        }
    }

    println!("🔌 WS disconnected");
}

/// Broadcasts a JSON message to all connected clients.
fn broadcast_json(clients: &[UnboundedSender<Message>], json: &Value) {
    let msg = Message::Text(json.to_string());
    for client in clients.iter() {
        let _ = client.send(msg.clone());
    }
}