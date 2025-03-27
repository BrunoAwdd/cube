use crate::state::AppState;
use axum::extract::ws::{Message, WebSocketUpgrade, WebSocket};
use axum::{extract::State, response::IntoResponse, routing::get, Router};
use std::sync::Arc;
use futures_util::{StreamExt, SinkExt};
use tokio::sync::{Mutex, mpsc::UnboundedSender};

pub type Clients = Arc<Mutex<Vec<UnboundedSender<Message>>>>;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>, // <- usa AppState aqui
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    state.ws_state.lock().await.push(tx);

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let _ = sender.send(msg).await;
        }
    });

    while let Some(Ok(_)) = receiver.next().await {
        // Ignorar mensagens recebidas do client (se desejar)
    }
}

pub fn create_ws_router() -> Router<Arc<AppState>> {
    Router::new().route("/ws", get(ws_handler))
}
