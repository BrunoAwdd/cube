//! # Thumbnails Handler
//!
//! This module provides endpoints for uploading and listing photo thumbnails.
//!
//! ## Endpoints
//! - **upload_thumbs_handler**: Receives a list of thumbnails in base64, saves them to disk, and updates the database.
//! - **list_thumbs_handler**: Lists all thumbnails available in the `.thumbs` directory, returning their metadata.
//!
//! ## Structures
//! - `ThumbPayload`: Payload for uploading a thumbnail (id, name, size, hash, status, thumb_base64, modified_at).
//! - `Photo`: Metadata returned when listing thumbnails (id, url, name, size, status).

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use base64::{engine::general_purpose, Engine};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;
use std::{fs, path::PathBuf, sync::Arc};

use tauri::Emitter;
use tauri::Listener;

use crate::state::{AppState, DbRequest};

/// Payload for uploading a thumbnail.
#[derive(Deserialize)]
pub struct ThumbPayload {
    id: String,
    name: String,
    size: String,
    hash: String,
    status: String,
    thumb_base64: String,
    modified_at: Option<DateTime<chrono::Utc>>,
}

/// Metadata for a photo thumbnail.
#[derive(Serialize)]
pub struct Photo {
    pub id: String,
    pub url: String,
    pub name: String,
    pub size: String,
    pub status: String,
}

/// Receives a list of thumbnails, saves them to disk, and updates the database.
///
/// # Flow
/// - Ensures the `.thumbs` directory exists.
/// - Decodes each thumbnail from base64 and saves it as a JPEG file.
/// - Inserts or updates the thumbnail metadata in the database.
/// - Returns a success message.
pub async fn upload_thumbs_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Vec<ThumbPayload>>,
) -> impl IntoResponse {
    let thumb_dir = PathBuf::from(".thumbs");
    if !thumb_dir.exists() {
        if let Err(e) = fs::create_dir_all(&thumb_dir) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erro ao criar diretório .thumbs: {e}"),
            );
        }
    }

    for item in &payload {
        let file_path = thumb_dir.join(format!("{}.jpg", item.hash));

        // Grava o arquivo da thumb
        if let Ok(bytes) = general_purpose::STANDARD.decode(&item.thumb_base64) {
            if let Err(e) = fs::write(&file_path, bytes) {
                eprintln!("Erro ao salvar thumb {}: {e}", file_path.display());
                continue;
            }
        }

        // Monta e envia comando SQL via canal
        let sql =
            "INSERT OR REPLACE INTO uploads (hash, filename, size) VALUES (?, ?, ?)".to_string();
        let values = vec![json!(item.hash), json!(item.name), json!(item.size)];

        // Pode ignorar o resultado se for apenas "fire-and-forget"
        let (_tx, _rx) = tokio::sync::oneshot::channel();
        if let Err(e) = state
            .db_tx
            .send(DbRequest {
                sql,
                values,
                respond_to: _tx,
            })
            .await
        {
            eprintln!("Erro ao inserir thumb no DB: {e}");
        }
    }

    (
        StatusCode::OK,
        "Thumbs recebidos e processados com sucesso".to_string(),
    )
}

/// Lists all thumbnails available in the `.thumbs` directory.
///
/// # Flow
/// - Reads thumbnail metadata from the database.
/// - Checks if the corresponding JPEG file exists in `.thumbs`.
/// - Returns a list of `Photo` objects as JSON.
pub async fn list_thumbs_handler(window: tauri::Window) -> impl IntoResponse {
    let (tx, rx): (tokio::sync::oneshot::Sender<String>, _) = tokio::sync::oneshot::channel();
    let event_id = format!("list_thumbs_response_{}", uuid::Uuid::new_v4());

    // Ouve a resposta do SELECT
    window.once(&event_id, move |event| {
        let payload_str = event.payload();
        let _ = tx.send(payload_str.to_string());
    });

    // Dispara o SELECT
    let query_payload = json!({
        "db": "sqlite:uploads.db",
        "sql": "SELECT hash, filename, size FROM uploads",
        "values": [],
        "response": event_id
    });

    if let Err(_e) = window.emit("plugin:sql|select", query_payload) {
        return Json(vec![]); // erro ao emitir, retorna vazio
    }

    // Espera resultado
    let data = match tokio::time::timeout(std::time::Duration::from_secs(2), rx).await {
        Ok(Ok(payload)) => payload,
        _ => return Json(vec![]), // erro ou timeout
    };

    // Parse resultado
    let rows: Vec<serde_json::Value> = match serde_json::from_str(&data) {
        Ok(rows) => rows,
        Err(_) => return Json(vec![]),
    };

    // Verifica se os arquivos existem
    let thumb_dir = Path::new(".thumbs");
    let filtered = rows
        .into_iter()
        .filter_map(|row| {
            let hash = row.get("hash")?.as_str()?.to_string();
            let filename = row.get("filename")?.as_str()?.to_string();
            let size = row.get("size")?.as_str()?.to_string();
            let path = thumb_dir.join(format!("{hash}.jpg"));

            if path.exists() {
                Some(Photo {
                    id: hash.clone(),
                    url: format!("/thumbs/{}.jpg", hash),
                    name: filename,
                    size,
                    status: "uploading".to_string(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Json(filtered)
}
