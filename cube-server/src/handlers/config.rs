use axum::{extract::State, Json};
use serde::Deserialize;
use std::{sync::Arc, path::PathBuf};
use chrono::Utc;
use chrono::Datelike;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct ConfigPayload {
    upload_dir: Option<String>,
}

pub async fn set_config_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ConfigPayload>,
) -> String {
    // Usa o valor fornecido ou gera o padrão
    let new_dir = payload.upload_dir.unwrap_or_else(|| {
        let now = Utc::now();
        format!("Imagens/bruno/{}/{}", now.year(), format!("{:02}", now.month()))
    });

    // Cria o diretório, se necessário
    let path = PathBuf::from(&new_dir);
    if let Err(e) = tokio::fs::create_dir_all(&path).await {
        return format!("❌ Erro ao criar diretório {}: {}", new_dir, e);
    }

    // Atualiza o estado global
    let mut dir = state.upload_dir.write().await;
    *dir = new_dir.clone();

    format!("📂 Diretório de upload definido para: {}", new_dir)
}
