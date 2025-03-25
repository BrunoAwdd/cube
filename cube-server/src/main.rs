mod state;
mod handlers;
mod utils;

use axum::{routing::{get, post}, Router};
use handlers::upload::upload_handler;
use handlers::auth::{generate_code_handler, auth_handler};
use handlers::upload_raw::upload_raw_handler;
use state::AppState;
use dirs::picture_dir;
use local_ip_address::local_ip;
use std::{sync::Arc, net::SocketAddr};
use tokio::fs;
use tower_http::cors::{CorsLayer, Any};
use rusqlite::Connection;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let default_dir = picture_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("uploads"))
        .to_str()
        .unwrap()
        .to_string();

    fs::create_dir_all(&default_dir).await.unwrap();

    let conn = Connection::open("uploads.db").expect("Falha ao abrir DB");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS uploads (
            hash TEXT PRIMARY KEY,
            filename TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS auth_codes (
            code TEXT PRIMARY KEY,
            created_at TIMESTAMP,
            ip TEXT
        )",
        [],
    ).unwrap();

    let state = AppState {
        upload_dir: default_dir,
        db: Arc::new(Mutex::new(conn)),
    };

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    let app = Router::new()
        .route("/upload", post(upload_handler))
        .route("/upload_raw", post(upload_raw_handler))
        .route("/generate_code", get(generate_code_handler))
        .route("/auth", post(auth_handler))
        .route("/ping", get(|| async { "pong" }))
        .with_state(Arc::new(state))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    if let Ok(ip) = local_ip() {
        println!("📱 Escaneie: http://{}:8080", ip);
    }

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await.unwrap();
}
