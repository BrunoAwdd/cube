use axum::{extract::{State, Json}, response::IntoResponse};
use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose, Engine};
use std::{fs, path::PathBuf, sync::Arc};
use chrono::DateTime;
use std::path::Path;


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

#[derive(Serialize)]
pub struct Photo {
    pub id: String,
    pub url: String,
    pub name: String,
    pub size: String,
    pub status: String,
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
            "INSERT OR REPLACE INTO uploads (hash, filename, size) VALUES (?1, ?2, ?3)",
            [&item.hash, &item.name, &item.size],
        ).unwrap();
    }

    tx.commit().unwrap();
    "Thumbs recebidos e processados com sucesso".to_string()
}

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
            let size: String = row.get(2)?; // ← capturando o tamanho real da foto
            Ok((hash, filename, size))
        })
        .unwrap();

    for row in rows.flatten() {
        let (hash, filename, size) = row;
        let path = thumb_dir.join(format!("{}.jpg", hash));

        if path.exists() {
            result.push(Photo {
                id: hash.clone(),
                url: format!("/thumbs/{}.jpg", hash),
                name: filename,
                size, // ← agora o tamanho correto vindo do Flutter
                status: "uploading".to_string(), // ← pode ajustar futuramente
            });
        }
    }
    Json(result)
}

