use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    Json as AxumJson,
};
use rand::{distributions::Alphanumeric, Rng};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use axum::extract::ws::Message;

use crate::state::AppState;
use local_ip_address::local_ip;
use serde_json::json;

/// # Authentication Module
///
/// This module implements a simple code-based authentication flow for opening a session on the server.
/// The flow consists of two main endpoints:
///
/// - **Code Generation (`generate_code_handler`)**: Generates a random 6-character code, saves it in the database along with the server's IP, and returns it to the client. The code expires in 60 seconds.
/// - **Authentication (`auth_handler`)**: Receives a code and username, validates the code in the database, generates a UUID token for the session, and notifies all connected WebSocket clients with the new token.
///
/// ## Structures
/// - `CodeResponse`: Response when generating a code (code, ip, expires_in).
/// - `AuthRequest`: Payload for authentication (code, username).
/// - `AuthResponse`: Response when authenticating (token).
///
/// ## Authentication Flow
/// 1. The client requests an authentication code.
/// 2. The server generates and returns the code, IP, and expiration time.
/// 3. The client sends the code and username for authentication.
/// 4. If the code is valid, the server generates a token, saves it in the database, and notifies via WebSocket.
/// 5. The client receives the token for use in subsequent requests.
///
/// ## Notes
/// - The code does not check for expiration, only existence.
/// - The returned IP is always the server's, not the client's.
/// - All tokens and codes are stored in SQLite.
/// - Real-time notifications are sent via WebSocket.

/// Response when generating an authentication code.
#[derive(Serialize)]
pub struct CodeResponse {
    pub code: String,
    pub ip: String,
    pub expires_in: u64,
}

/// Payload for authentication.
#[derive(Deserialize)]
pub struct AuthRequest {
    pub code: String,
    pub username: String,
}

/// Response when authenticating.
#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}

/// Generates a 6-character code, saves it in the database, and returns it to the client.
///
/// # Flow
/// - Generates a random code.
/// - Saves it in the database with timestamp and IP.
/// - Returns JSON with code, IP, and expiration time.
pub async fn generate_code_handler(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let code = generate_code(6);
    let expires_in = 60;

    let ip = match local_ip() {
        Ok(ip) => ip.to_string(),
        Err(_) => "127.0.0.1".to_string(),
    };


    let now = Utc::now();

    let db = state.db.lock().await;
    let _ = db.execute(
        "INSERT OR REPLACE INTO auth_codes (code, created_at, ip) VALUES (?1, ?2, ?3)",
        params![code, now.to_rfc3339(), ip],
    );

    AxumJson(CodeResponse {
        code,
        ip,
        expires_in,
    })
}

/// Authenticates the user using code and username, returns a session token.
///
/// # Flow
/// - Validates code in the database.
/// - Generates a UUID token.
/// - Saves the token in the database.
/// - Notifies WebSocket clients.
/// - Returns token in JSON.
pub async fn auth_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthRequest>,
) -> impl IntoResponse {
    let db = state.db.lock().await;

    println!("⚠️ Autenticando com o código {}", payload.code);

    let result: rusqlite::Result<String> = db.query_row(
        "SELECT ip FROM auth_codes WHERE code = ?1",
        [payload.code.clone()],
        |row| row.get(0),
    );

    let ip: String = match result {
        Ok(ip) => ip,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Código inválido").into_response(),
    };

    let token = Uuid::new_v4().to_string();
    let now = Utc::now();

    // Salve the new token in the database
    let _ = db.execute(
        "INSERT INTO tokens (token, username, ip, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![token, payload.username, ip, now.to_rfc3339()],
    );

    drop(db); // Release the lock before sending the message

    // 🔔 Notificar via WebSocket
    let msg = json!({
        "token": token
    }).to_string();

    let clients = state.ws_state.lock().await;
    for tx in clients.iter() {
        let _ = tx.send(Message::Text(msg.clone()));
    }

    (StatusCode::OK, AxumJson(AuthResponse { token })).into_response()
}

/// Generates an alphanumeric code of `len` characters.
fn generate_code(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
