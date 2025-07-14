use chrono::{DateTime, Datelike, Utc};
use std::path::PathBuf;
use tokio::fs;

/// Builds and ensures the output path for a file, organizing by username and optionally by year/month.
///
/// # Arguments
/// * `base` - The base directory as a string.
/// * `username` - The username to include in the path.
/// * `filename` - The name of the file to be saved.
/// * `modified_at` - Optional modification date to organize files by year and month.
///
/// # Returns
/// A `PathBuf` representing the full output path where the file should be saved. The function also ensures
/// that the directory exists, creating it if necessary.
///
/// # Example
/// ```
/// let path = get_output_path("AppData/cube", "alice", "photo.raw", Some(Utc::now())).await;
/// ```
pub async fn get_output_path(
    base: &str,
    username: &str,
    filename: &str,
    modified_at: Option<DateTime<Utc>>,
) -> PathBuf {
    let base_path = PathBuf::from(base);
    let path = if let Some(modified) = modified_at {
        let dir = base_path
            .join(username)
            .join(modified.year().to_string())
            .join(format!("{:02}", modified.month()));
        fs::create_dir_all(&dir)
            .await
            .expect("Failed to create directory");
        dir.join(filename)
    } else {
        let dir = base_path.join(username);
        fs::create_dir_all(&dir)
            .await
            .expect("Failed to create directory");
        dir.join(filename)
    };

    path
}
