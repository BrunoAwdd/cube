// src/handlers/upload_handler.rs
use axum::{extract::{State, Multipart}, debug_handler};
use std::path::PathBuf;
use std::sync::Arc;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rusqlite::params;

use crate::state::AppState;
use crate::utils::{path::get_output_path, file::save_file, hash::compute_hash};

#[debug_handler]
pub async fn upload_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> String {
    let mut username = "bruno".to_string();
    let mut file_bytes: Option<Bytes> = None;
    let mut modified_at: Option<DateTime<Utc>> = None;
    let mut filename: Option<String> = None;

    while let Some(field_result) = multipart.next_field().await.transpose() {
        let field = match field_result {
            Ok(f) => f,
            Err(err) => {
                eprintln!("Erro ao ler multipart field: {:?}", err);
                return "Erro ao processar campo do formulário.".to_string();
            }
        };

        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "modified_at" => {
                if let Ok(text) = field.text().await {
                    modified_at = DateTime::parse_from_rfc3339(&text)
                        .or_else(|_| DateTime::parse_from_str(&text, "%Y-%m-%dT%H:%M:%S%.3f"))
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc));
                }
            }
            "username" => {
                username = field.text().await.unwrap_or(username);
            }
            "file" => {
                if let Some(content_type) = field.content_type() {
                    println!("📦 Tipo MIME: {}", content_type);
                }

                let original_filename = field.file_name().map(|f| f.to_string());

                let ext = original_filename
                    .as_ref()
                    .map(|name| {
                        let path = PathBuf::from(name);
                        path.extension()
                            .map(|ext| ext.to_string_lossy().to_string())
                    })
                    .flatten();

                let safe_filename = match ext {
                    Some(ext) => format!("{}_upload.{}", Uuid::new_v4(), ext),
                    None => format!("{}_upload", Uuid::new_v4()),
                };

                filename = Some(original_filename.unwrap_or(safe_filename));
                file_bytes = field.bytes().await.ok();
            }
            _ => {}
        }
    }

    let data = match file_bytes {
        Some(d) => d,
        None => return "Nenhum arquivo enviado.".to_string(),
    };

    let filename = filename.unwrap_or_else(|| format!("{}_upload", Uuid::new_v4()));
    let hash = compute_hash(&data);

    let db = state.db.lock().await;
    let exists: bool = db
        .query_row("SELECT EXISTS(SELECT 1 FROM uploads WHERE hash = ?1)", [hash.clone()], |row| row.get(0))
        .unwrap_or(false);

    if exists {
        println!("Arquivo já existente: {}", filename);
        return "Arquivo já existente, ignorado.".to_string();
    }

    let path = get_output_path(&state.upload_dir, &username, &filename, modified_at).await;
    println!("modified_at recebido: {:?}", modified_at);

    save_file(&path, &data).await;

    db.execute(
        "INSERT INTO uploads (hash, filename) VALUES (?1, ?2)",
        params![hash, filename],
    ).unwrap();

    println!("Recebido e salvo: {}", path.to_string_lossy());
    "Upload finalizado!".to_string()
}
