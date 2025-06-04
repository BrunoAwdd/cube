use std::path::PathBuf;
use std::sync::Arc;
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub async fn start_tcp_server(shared_state: Arc<crate::state::AppState>) {
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();
    println!("TCP mock listening on 127.0.0.1:7878");

    loop {
        match listener.accept().await {
            Ok((mut socket, addr)) => {
                println!("📡 TCP connection from: {}", addr);

                let state = shared_state.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(&mut socket).await {
                        eprintln!("TCP client error: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("TCP accept error: {}", e),
        }
    }
}

async fn handle_client(socket: &mut tokio::net::TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = [0; 1024];
    let n = socket.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }

    let msg = String::from_utf8_lossy(&buf[..n]);
    println!("🔹 TCP received: {}", msg.trim());

    let response = match msg.trim() {
        "CONFIG" => handle_config_command().await?,
        _ => r#"{"error":"Unknown command"}"#.to_string(),
    };

    socket.write_all(response.as_bytes()).await?;
    Ok(())
}

async fn handle_config_command() -> Result<String, Box<dyn std::error::Error>> {
    let base_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("C:\\Temp"))
        .join("Cube");
    let user = "bruno";

    let dcim_dir = base_dir.join(&user).join("dcim");
    let dcim_thumbs = dcim_dir.join("thumbs");

    let downloads_dir = base_dir.join(&user).join("downloads");
    let downloads_thumbs = downloads_dir.join("thumbs");

    create_dir_if_not_exists(&dcim_thumbs).await?;
    create_dir_if_not_exists(&downloads_thumbs).await?;

    let config_json = serde_json::json!({
        "User": user,
        "MainDirectory": base_dir.to_string_lossy(),
    });

    println!("🔹 TCP config response: {}", config_json);

    Ok(config_json.to_string())
}

async fn create_dir_if_not_exists(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if !tokio::fs::try_exists(path).await? {
        tokio::fs::create_dir_all(path).await?;
    }
    Ok(())
}