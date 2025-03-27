use axum::{extract::{State}, Json};
use serde::Deserialize;
use std::sync::Arc;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ConfigPayload {
    upload_dir: Option<String>,
}

pub async fn set_config_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ConfigPayload>,
) -> String {
    if let Some(new_dir) = payload.upload_dir {
        let mut dir = state.upload_dir.write().await;
        *dir = new_dir.clone();
        return format!("📂 Novo diretório definido: {}", new_dir);
    }

    "Nenhuma configuração atualizada.".to_string()
}
