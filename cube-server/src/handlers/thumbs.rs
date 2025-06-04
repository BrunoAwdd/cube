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

use axum::{extract::{State, Json}, response::IntoResponse};
use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose, Engine};
use std::{fs, path::PathBuf, sync::Arc};
use chrono::DateTime;
use std::path::Path;


use crate::state::AppState;

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
            return format!("Erro ao criar diretório .thumbs: {e}");
        }
    }

    let mut db = state.db.lock().await;
    let tx = db.transaction().expect("Failed to create transaction");

    for item in payload {
        let file_path = thumb_dir.join(format!("{}.jpg", item.hash));

        // Save thumbnail to disk
        if let Ok(bytes) = general_purpose::STANDARD.decode(&item.thumb_base64) {
            if let Err(e) = fs::write(&file_path, bytes) {
                eprintln!("Erro ao salvar thumb {}: {e}", file_path.display());
                continue;
            }
        }

        // Insert on database
        tx.execute(
            "INSERT OR REPLACE INTO uploads (hash, filename, size) VALUES (?1, ?2, ?3)",
            [&item.hash, &item.name, &item.size],
        ).expect("Failed to insert or update file in database");
    }

    tx.commit().expect("Failed to commit transaction");
    "Thumbs recebidos e processados com sucesso".to_string()
}

/// Lists all thumbnails available in the `.thumbs` directory.
///
/// # Flow
/// - Reads thumbnail metadata from the database.
/// - Checks if the corresponding JPEG file exists in `.thumbs`.
/// - Returns a list of `Photo` objects as JSON.
pub async fn list_thumbs_handler(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<Photo>> {
    let mut result = Vec::new();
    let thumb_dir = Path::new(".thumbs");

    let db = state.db.lock().await;
    let mut stmt = db
        .prepare("SELECT hash, filename, size FROM uploads")
        .unwrap();

    let rows = stmt
        .query_map([], |row| {
            let hash: String = row.get(0)?;
            let filename: String = row.get(1)?;
            let size: String = row.get(2)?;
            Ok((hash, filename, size))
        })
        .expect("Failed to query uploads");

    for row in rows.flatten() {
        let (hash, filename, size) = row;
        let path = thumb_dir.join(format!("{}.jpg", hash));

        if path.exists() {
            result.push(Photo {
                id: hash.clone(),
                url: format!("/thumbs/{}.jpg", hash),
                name: filename,
                size,
                status: "uploading".to_string(),
            });
        }
    }
    Json(result)
}

