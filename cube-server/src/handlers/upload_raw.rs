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
    http::HeaderMap,
    response::IntoResponse,
};
use axum::debug_handler;
use std::{sync::Arc};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rusqlite::params;

use crate::state::AppState;
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
#[debug_handler]
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
        Err(_) => return "Error reading file".to_string(),
    };

    let hash = compute_hash(&data);

    let db = state.db.lock().await;
    let exists: bool = db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM uploads WHERE hash = ?1)",
            [hash.clone()],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if exists {
        println!("📦 File {} already exists", hash);
        return "The file already exists".to_string();
    }

    let dir = state.upload_dir.read().await;
    let path = get_output_path(&dir, &username, &filename, modified_at).await;

    save_file(&path, &data).await;

    db.execute(
        "INSERT INTO uploads (hash, filename) VALUES (?1, ?2)",
        params![hash, filename],
    )
    .unwrap();

    println!("✅ Received and Saved: {}", path.to_string_lossy());

    // Send notification to WebSocket clients
    let confirmation = serde_json::json!({
        "event": "copied",
        "hash": hash,
        "status": "success",
        "path": path.to_string_lossy(),
    });

    let clients = state.ws_state.lock().await;
    for client in clients.iter() {
        let _ = client.send(Message::Text(confirmation.to_string()));
    }

    "Upload Ended!".to_string()
}
