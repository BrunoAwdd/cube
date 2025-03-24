use std::path::PathBuf;
use chrono::{DateTime, Utc, Datelike};
use tokio::fs;

pub async fn get_output_path(
    base: &str,
    username: &str,
    filename: &str,
    modified_at: Option<DateTime<Utc>>,
) -> PathBuf {
    let base_path = PathBuf::from(base);
    let path = if let Some(modified) = modified_at {
        let dir = base_path.join(username).join(modified.year().to_string()).join(format!("{:02}", modified.month()));
        fs::create_dir_all(&dir).await.unwrap();
        dir.join(filename)
    } else {
        let dir = base_path.join(username);
        fs::create_dir_all(&dir).await.unwrap();
        dir.join(filename)
    };

    path
}
