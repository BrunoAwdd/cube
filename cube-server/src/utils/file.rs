use std::path::PathBuf;
use tokio::fs;

pub async fn save_file(path: &PathBuf, data: &[u8]) {
    fs::write(path, data).await.unwrap();
}
