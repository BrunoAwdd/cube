//! # RAW Upload Handler
//!
//! This module provides an endpoint for uploading RAW files to the server.
//!
//! ## Endpoint
//! - **upload_raw_handler**: Receives a file upload (with metadata in headers), saves it to disk, updates the database, and notifies connected WebSocket clients.
//!
//! ## Flow
//! 1. Extracts username, filename, and modification date from HTTP headers.
//! 2. Reads the file body and computes its hash.
//! 3. Checks if the file (by hash) already exists in the database; if so, ignores the upload.
//! 4. Determines the output path based on user and date, and saves the file.
//! 5. Inserts the file metadata into the database.
//! 6. Notifies all connected WebSocket clients about the new upload.
//! 7. Returns a success message.

use axum::{
    body::{to_bytes, Body},
    extract::{ws::Message, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};

use chrono::{DateTime, Utc};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::{AppState, DbRequest};
use crate::utils::{file::save_file, hash::compute_hash, path::get_output_path};

/// Handles RAW file uploads.
///
/// # Flow
/// - Extracts metadata from headers.
/// - Reads and saves the file.
/// - Checks for duplicates by hash.
/// - Updates the database.
/// - Notifies WebSocket clients.
/// - Returns a status message.

pub async fn upload_raw_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Body,
) -> impl IntoResponse {
    let username = headers
        .get("X-Username")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default")
        .to_string();

    let filename = headers
        .get("X-Filename")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}_upload", Uuid::new_v4()));

    let modified_at = headers
        .get("X-Modified-At")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let data = match to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes.to_vec(),
        Err(_) => return (StatusCode::BAD_REQUEST, "Erro ao ler o arquivo".to_string()),
    };

    let hash = compute_hash(&data);

    // Consulta se já existe
    let (tx, rx) = tokio::sync::oneshot::channel();
    let sql = "SELECT EXISTS(SELECT 1 FROM uploads WHERE hash = ?)".to_string();
    let values = vec![json!(hash)];

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
            format!("Erro ao consultar DB: {e}"),
        );
    }

    let exists = match tokio::time::timeout(std::time::Duration::from_secs(2), rx).await {
        Ok(Ok(payload)) => {
            payload
                .get(0)
                .and_then(|row| row.as_object())
                .and_then(|row| row.values().next()) // Pega a primeira coluna da row
                .and_then(|v| v.as_i64())
                .unwrap_or(0)
                == 1
        }
        _ => false,
    };

    if exists {
        println!("📦 File {} already exists", hash);
        return (StatusCode::OK, "The file already exists".to_string());
    }

    // Salva o arquivo
    let dir = state.upload_dir.read().await;
    let path = get_output_path(&dir, &username, &filename, modified_at).await;
    if let Err(e) = save_file(&path, &data).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erro ao salvar arquivo: {e}"),
        );
    }

    // Insere no banco
    let (_tx, _rx) = tokio::sync::oneshot::channel(); // Fire-and-forget
    let insert_sql = "INSERT INTO uploads (hash, filename) VALUES (?, ?)".to_string();
    let insert_values = vec![json!(hash), json!(filename)];

    if let Err(e) = state
        .db_tx
        .send(DbRequest {
            sql: insert_sql,
            values: insert_values,
            respond_to: _tx,
        })
        .await
    {
        eprintln!("Erro ao inserir no DB: {e}");
    }

    println!("✅ Received and Saved: {}", path.to_string_lossy());

    // Notifica via WebSocket
    let confirmation = json!({
        "event": "copied",
        "hash": hash,
        "status": "success",
        "path": path.to_string_lossy(),
    });

    let clients = state.ws_state.lock().await;
    for client in clients.iter() {
        let _ = client.send(Message::Text(confirmation.to_string()));
    }

    (StatusCode::OK, "Upload Ended!".to_string())
}
