use axum::{
    extract::{State, RawBody},
    http::HeaderMap,
    debug_handler,
    body::Bytes,
};
use std::{path::PathBuf, sync::Arc};
use chrono::{DateTime, Utc};
use rusqlite::params;
use uuid::Uuid;

use crate::{
    state::AppState,
    utils::{hash::compute_hash, path::get_output_path, file::save_file},
};

#[debug_handler]
pub async fn upload_raw_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    RawBody(body): RawBody,
) -> String {
    // Leitura do corpo
    let data = match hyper::body::to_bytes(body).await {
        Ok(d) => d,
        Err(_) => return "Erro ao ler corpo".to_string(),
    };

    // Headers
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

    let username = headers
        .get("X-Username")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("bruno")
        .to_string();

    // Hash
    let hash = compute_hash(&data);

    // DB check
    let db = state.db.lock().await;
    let exists: bool = db
        .query_row("SELECT EXISTS(SELECT 1 FROM uploads WHERE hash = ?1)", [hash.clone()], |row| row.get(0))
        .unwrap_or(false);

    if exists {
        println!("Arquivo já existente: {}", filename);
        return "Arquivo já existente, ignorado.".to_string();
    }

    let path = get_output_path(&state.upload_dir, &username, &filename, modified_at).await;

    save_file(&path, &data).await;

    db.execute(
        "INSERT INTO uploads (hash, filename) VALUES (?1, ?2)",
        params![hash, filename],
    ).unwrap();

    println!("Recebido e salvo: {}", path.to_string_lossy());
    "Upload finalizado!".to_string()
}
