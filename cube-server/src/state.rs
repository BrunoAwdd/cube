use std::sync::Arc;
use rusqlite::Connection;
use tokio::sync::{Mutex, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub upload_dir: Arc<RwLock<String>>,
    pub db: Arc<Mutex<Connection>>,
}
