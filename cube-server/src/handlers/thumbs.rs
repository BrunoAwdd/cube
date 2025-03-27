use axum::{extract::{State, Json}, response::IntoResponse};
use serde::Deserialize;
use base64::{engine::general_purpose, Engine};
use std::{fs, path::PathBuf, sync::Arc};
use chrono::DateTime;

use crate::state::AppState;

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
    let tx = db.transaction().unwrap();

    for item in payload {
        let file_path = thumb_dir.join(format!("{}.jpg", item.hash));

        // Salvar o thumbnail no disco
        if let Ok(bytes) = general_purpose::STANDARD.decode(&item.thumb_base64) {
            if let Err(e) = fs::write(&file_path, bytes) {
                eprintln!("Erro ao salvar thumb {}: {e}", file_path.display());
                continue;
            }
        }

        // Inserir ou atualizar no banco de dados
        tx.execute(
            "INSERT OR IGNORE INTO uploads (hash, filename) VALUES (?1, ?2)",
            [&item.hash, &item.name],
        ).unwrap();
    }

    tx.commit().unwrap();
    "Thumbs recebidos e processados com sucesso".to_string()
}