use crate::ws::Clients;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock};

/// Global application state shared across handlers.
///
/// - `upload_dir`: The current upload directory, protected by an async RwLock.
/// - `db`: The SQLite database connection, protected by an async Mutex.
/// - `ws_state`: The list of connected WebSocket clients.
#[derive(Clone)]
pub struct AppState {
    pub upload_dir: Arc<RwLock<String>>,
    pub ws_state: Clients,
    pub db_tx: tokio::sync::mpsc::Sender<DbRequest>,
}

pub struct DbRequest {
    pub sql: String,
    pub values: Vec<Value>,
    pub respond_to: oneshot::Sender<Value>,
}
