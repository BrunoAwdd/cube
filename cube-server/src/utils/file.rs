use std::path::PathBuf;
use tokio::fs;

/// Saves binary data to the specified file path asynchronously.
///
/// # Arguments
/// * `path` - The path where the file will be saved.
/// * `data` - The binary data to write.
///
/// # Panics
/// Panics if the file cannot be written.
///
/// # Example
/// ```
/// let path = PathBuf::from("output.raw");
/// save_file(&path, b"file content").await;
/// ```
pub async fn save_file(path: &PathBuf, data: &[u8]) {
    fs::write(path, data).await.unwrap();
}