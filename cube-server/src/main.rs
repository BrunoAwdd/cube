use axum::{
    extract::{Multipart, State},
    routing::{get, post},
    Router,
};
use dirs::picture_dir;
use local_ip_address::local_ip;
use std::{
    net::SocketAddr,
    sync::Arc,
};
use tokio::fs;
use tower_http::cors::{CorsLayer, Any};
use uuid::Uuid;
use rusqlite::{params, Connection};
use sha2::{Sha256, Digest};
use tokio::sync::Mutex;
use chrono::{DateTime, Utc, Datelike};
use std::path::PathBuf;

#[derive(Clone)]
struct AppState {
    upload_dir: String,
    db: Arc<Mutex<Connection>>,
}

#[tokio::main]
async fn main() {
    let default_dir = picture_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("uploads"))
        .to_str()
        .unwrap()
        .to_string();

    fs::create_dir_all(&default_dir).await.unwrap();

    // Cria o banco de dados SQLite
    let conn = Connection::open("uploads.db").expect("Falha ao abrir DB");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS uploads (
            hash TEXT PRIMARY KEY,
            filename TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).unwrap();

    let state = AppState {
        upload_dir: default_dir,
        db: Arc::new(Mutex::new(conn)), // <- agora sim é Mutex<Connection>
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/upload", post(upload_handler))
        .route("/ping", get(|| async { "pong" }))
        .with_state(Arc::new(state))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    if let Ok(ip) = local_ip() {
        println!("📱 Escaneie: http://{}:8080", ip);
    } else {
        println!("Servidor rodando em http://{}", addr);
    }

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}


#[axum::debug_handler]
async fn upload_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> String {
    let mut modified_at: Option<DateTime<Utc>> = None;
    let mut filename = None;
    let mut file_bytes = None;
    let mut username = "bruno".to_string();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "modified_at" => {
                if let Ok(text) = field.text().await {
                    println!("modified_at recebido: {}", text);
                    modified_at = DateTime::parse_from_rfc3339(&text)
                        .or_else(|_| DateTime::parse_from_str(&text, "%Y-%m-%dT%H:%M:%S%.3f"))
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc));
                }
            },
            "file" => {
                filename = field
                    .file_name()
                    .map(|f| f.to_string())
                    .or_else(|| Some(format!("{}_upload.jpg", Uuid::new_v4())));
                file_bytes = Some(field.bytes().await.unwrap());
            },
            "username" => {
                if let Ok(text) = field.text().await {
                    username = text;
                }
            },
            _ => {}
        }
    }

    // Se não houver arquivo, retorna erro simples
    let data = match file_bytes {
        Some(data) => data,
        None => return "Nenhum arquivo enviado.".to_string(),
    };

    let filename = filename.unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let hash = format!("{:x}", hasher.finalize());

    let db = state.db.lock().await;
    let exists: bool = db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM uploads WHERE hash = ?1)",
            [hash.clone()],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if exists {
        println!("Arquivo já existente: {}", filename);
        return "Arquivo já existente, ignorado.".to_string();
    }

    let base_path = PathBuf::from(&state.upload_dir);
    let path = if let Some(modified) = modified_at {
        let year = modified.year().to_string();
        let month = format!("{:02}", modified.month());

        let dir = base_path.join(&username).join(&year).join(&month);
        fs::create_dir_all(&dir).await.unwrap();

        dir.join(&filename).to_string_lossy().to_string()
    } else {
        let dir = base_path.join(&username);
        fs::create_dir_all(&dir).await.unwrap();

        dir.join(&filename).to_string_lossy().to_string()
    };

    println!("modified_at recebido: {:?}", modified_at);

    fs::write(&path, &data).await.unwrap();

    db.execute(
        "INSERT INTO uploads (hash, filename) VALUES (?1, ?2)",
        params![hash, filename],
    )
    .unwrap();

    println!("Recebido e salvo: {}", path);
    "Upload finalizado!".to_string()
}

