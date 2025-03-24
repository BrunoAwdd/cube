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

use crate::state::AppState;
use local_ip_address::local_ip;

#[derive(Serialize)]
pub struct CodeResponse {
    pub code: String,
    pub ip: String,
    pub expires_in: u64,
}

#[derive(Deserialize)]
pub struct AuthRequest {
    pub code: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}

/// Gera um código de 6 caracteres e salva no banco
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

/// Recebe um código e gera token se válido
pub async fn auth_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AuthRequest>,
) -> impl IntoResponse {
    let db = state.db.lock().await;

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

    let _ = db.execute(
        "INSERT INTO tokens (token, ip, created_at) VALUES (?1, ?2, ?3)",
        params![token, ip, now.to_rfc3339()],
    );

    (StatusCode::OK, AxumJson(AuthResponse { token })).into_response()
}

/// Gera um código alfanumérico de `len` caracteres
fn generate_code(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
