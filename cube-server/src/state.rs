use std::sync::Arc;
use rusqlite::Connection;
use tokio::sync::{Mutex, RwLock};
use crate::ws::Clients;

/// Global application state shared across handlers.
///
/// - `upload_dir`: The current upload directory, protected by an async RwLock.
/// - `db`: The SQLite database connection, protected by an async Mutex.
/// - `ws_state`: The list of connected WebSocket clients.
#[derive(Clone)]
pub struct AppState {
    pub upload_dir: Arc<RwLock<String>>,
    pub db: Arc<Mutex<Connection>>,
    pub ws_state: Clients,
}