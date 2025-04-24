use crate::state::AppState;
use axum::extract::ws::{Message, WebSocketUpgrade, WebSocket};
use axum::{extract::State, response::IntoResponse, routing::get, Router};
use std::sync::Arc;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::{Mutex, mpsc::UnboundedSender};
use serde_json::Value;

pub type Clients = Arc<Mutex<Vec<UnboundedSender<Message>>>>;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    // Adiciona esse cliente na lista
    state.ws_state.lock().await.push(tx.clone());

    // Task para enviar mensagens para esse socket
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let _ = sender.send(msg).await;
        }
    });

    // Loop para escutar o que chega desse socket
    while let Some(Ok(Message::Text(text))) = receiver.next().await {
        println!("📩 WS recebeu: {}", text);

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

                                        println!("⬇️ Enviando download para {}", h);
                                        broadcast_json(&clients, &request);
                                    }
                                }
                            }
                        }

                        _ => println!("⚠️ Ação desconhecida: {}", action),
                    }
                }

            }
            Err(err) => {
                println!("❌ Erro ao decodificar JSON do WS: {err}");
            }
        }
    }

    println!("🔌 WS desconectado");
}
fn broadcast_json(clients: &[UnboundedSender<Message>], json: &Value) {
    let msg = Message::Text(json.to_string());
    for client in clients.iter() {
        let _ = client.send(msg.clone());
    }
}
