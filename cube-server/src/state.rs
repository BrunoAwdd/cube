use std::sync::Arc;
use rusqlite::Connection;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub upload_dir: String,
    pub db: Arc<Mutex<Connection>>,
}
