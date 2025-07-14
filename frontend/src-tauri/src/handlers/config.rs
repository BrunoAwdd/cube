/// # Configuration Handler
///
/// This module provides an endpoint to set or update the upload directory used by the server.
///
/// ## Endpoint
/// - **set_config_handler**: Receives an optional upload directory path. If not provided, generates a default path based on the current year and month. Ensures the directory exists and updates the global application state.
///
/// ## Structures
/// - `ConfigPayload`: Payload for configuration (optional `upload_dir`).
use serde::Deserialize;
use std::sync::Arc;
use whoami;

use crate::state::AppState;

/// Payload for configuration requests.
/// If `upload_dir` is not provided, a default directory is generated.
#[derive(Deserialize, Debug)]
pub struct ConfigPayload {
    upload_dir: Option<String>,
}

/// Sets the upload directory for the server.
///
/// # Flow
/// - Uses the provided directory or generates a default one based on the current year and month.
/// - Creates the directory if it does not exist.
/// - Updates the global application state with the new directory.
/// - Returns a message indicating the result.
///
/// # Returns
/// A string message indicating success or failure.
pub async fn set_config_handler(
    state: Arc<AppState>,
    payload: ConfigPayload,
) -> Result<String, String> {
    use std::path::PathBuf;

    println!("Setting configuration with payload: {:?}", payload);

    // Diretório interno fixo
    let internal_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("C:\\Temp"))
        .join("Cube")
        .join(whoami::username())
        .join("dcim");

    if let Err(e) = tokio::fs::create_dir_all(&internal_dir).await {
        return Err(format!("❌ Error creating internal directory: {}", e));
    }

    // Diretório de exportação configurável
    let export_dir = payload
        .upload_dir
        .clone()
        .unwrap_or_else(|| "C:\\Export".to_string());

    if let Err(e) = tokio::fs::create_dir_all(&export_dir).await {
        return Err(format!("❌ Error creating export directory: {}", e));
    }

    // Atualiza o estado global
    {
        let mut dir = state.upload_dir.write().await;
        *dir = internal_dir.to_string_lossy().to_string();
    }
    {
        let mut export = state.upload_dir.write().await;
        *export = export_dir.clone();
    }

    Ok(format!(
        "📂 Internal directory: {}\n📤 Export directory: {}",
        internal_dir.to_string_lossy(),
        export_dir
    ))
}
