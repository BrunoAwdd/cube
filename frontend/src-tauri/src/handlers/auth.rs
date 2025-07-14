use axum::extract::ws::Message;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    Json as AxumJson,
};
use chrono::Utc;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use tauri::Emitter;

use crate::state::{AppState, DbRequest};
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
pub async fn generate_code_handler(window: tauri::Window) -> Result<CodeResponse, String> {
    let now = Utc::now().to_rfc3339();
    let ip = local_ip_address::local_ip()
        .map_err(|e| e.to_string())?
        .to_string();
    let code = generate_code(6);

    let payload = json!({
        "db": "sqlite:uploads.db",
        "sql": "INSERT INTO auth_codes (code, ip, created_at) VALUES (?, ?, ?)
                ON CONFLICT(code) DO UPDATE SET ip = excluded.ip, created_at = excluded.created_at",
        "values": [code, ip, now]
    });

    window
        .emit("plugin:sql|execute", payload)
        .map_err(|e| format!("Erro ao executar comando SQL: {}", e))?;

    Ok(CodeResponse {
        code,
        ip,
        expires_in: 60,
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
    let code = payload.code.clone();
    let username = payload.username.clone();

    println!("⚠️ Autenticando com o código {}", code);

    // Canal para resposta do Tauri (IP buscado por código)
    let (tx, rx) = tokio::sync::oneshot::channel();

    let sql = "SELECT ip FROM auth_codes WHERE code = ?".to_string();
    let values = vec![json!(code)];

    if let Err(e) = state
        .db_tx
        .send(DbRequest {
            sql,
            values,
            respond_to: tx,
        })
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erro ao enviar consulta ao Tauri: {}", e),
        )
            .into_response();
    }

    let response = match tokio::time::timeout(std::time::Duration::from_secs(2), rx).await {
        Ok(Ok(json_data)) => json_data,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                "Código inválido ou timeout".to_string(),
            )
                .into_response()
        }
    };

    // Parse do IP
    let ip = response
        .get(0)
        .and_then(|row| row.get("ip"))
        .and_then(|ip| ip.as_str())
        .map(|s| s.to_string());

    let ip = match ip {
        Some(ip) => ip,
        None => return (StatusCode::UNAUTHORIZED, "Código inválido".to_string()).into_response(),
    };

    // Gera token
    let token = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    // Envia comando de insert para Tauri via canal
    let (tx, rx) = tokio::sync::oneshot::channel();
    let insert_sql =
        "INSERT INTO tokens (token, username, ip, created_at) VALUES (?, ?, ?, ?)".to_string();
    let insert_values = vec![json!(token), json!(username), json!(ip), json!(now)];

    if let Err(e) = state
        .db_tx
        .send(DbRequest {
            sql: insert_sql,
            values: insert_values,
            respond_to: tx,
        })
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erro ao inserir token: {}", e),
        )
            .into_response();
    }

    let _ = rx.await; // ignorar resposta

    // WebSocket
    let msg = json!({ "token": token }).to_string();
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
