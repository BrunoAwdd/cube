use axum::{
    extract::{State},
    http::{HeaderMap},
    body::Body,
};
use axum::response::IntoResponse;
use axum::debug_handler;
use std::{sync::Arc};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use axum::body::to_bytes;
use rusqlite::params;

use crate::state::AppState;
use crate::utils::{path::get_output_path, file::save_file, hash::compute_hash};

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
        Ok(bytes) => bytes.to_vec(), // ← agora é Vec<u8>
        Err(_) => return "Erro ao ler corpo".to_string(),
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
        println!("Arquivo já existente: {}", filename);
        return "Arquivo já existente, ignorado.".to_string();
    }

    let dir = state.upload_dir.read().await;
    let path = get_output_path(&dir, &username, &filename, modified_at).await;

    println!("modified_at recebido: {:?}", modified_at);

    save_file(&path, &data).await;

    db.execute(
        "INSERT INTO uploads (hash, filename) VALUES (?1, ?2)",
        params![hash, filename],
    )
    .unwrap();

    println!("Recebido e salvo: {}", path.to_string_lossy());
    "Upload finalizado!".to_string()
}
