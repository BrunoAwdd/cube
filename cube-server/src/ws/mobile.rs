use crate::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;

pub async fn ws_mobile_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    println!("📡 Conexão WebSocket MÓVEL iniciada!");
    ws.on_upgrade(move |socket| handle_mobile_socket(socket, state))
}

async fn handle_mobile_socket(mut stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    // Adiciona esse cliente à lista global
    state.ws_state.lock().await.push(tx.clone());

    // Envia mensagem de boas-vindas
    let _ = sender.send(Message::Text(r#"{"status":"connected"}"#.into())).await;

    // Task para escutar e enviar mensagens da fila
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let _ = sender.send(msg).await;
        }
    });

    // Ignora mensagens recebidas
    while let Some(Ok(_)) = receiver.next().await {}
}
