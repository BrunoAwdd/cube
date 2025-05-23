mod state;
mod handlers;
mod utils;
mod ws;

use axum::{routing::{get, post}, Router};
use handlers::auth::{generate_code_handler, auth_handler};
use handlers::upload_raw::upload_raw_handler;
use handlers::thumbs::{upload_thumbs_handler, list_thumbs_handler};
use handlers::config::set_config_handler;
use state::AppState;
use dirs::picture_dir;
use local_ip_address::local_ip;
use std::{sync::Arc, net::SocketAddr};
use tokio::fs;
use tower_http::cors::{CorsLayer, Any};
use rusqlite::Connection;
use ws::create_ws_router;
use tokio::sync::{Mutex, RwLock};
use tower_http::services::ServeDir;


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
            size TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tokens (
            token TEXT PRIMARY KEY,
            username TEXT,
            ip TEXT,
            created_at TIMESTAMP
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
        upload_dir: Arc::new(RwLock::new(default_dir)),
        db: Arc::new(Mutex::new(conn)),
        ws_state: Arc::new(Mutex::new(Vec::new())),
    };
    
    let static_files = ServeDir::new(".thumbs").append_index_html_on_directories(false);
    let thumbs_route = Router::new().nest_service("/thumbs", static_files);

    let shared_state = Arc::new(state);

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    let app = Router::new()
        .route("/upload_raw", post(upload_raw_handler))
        .route("/generate_code", get(generate_code_handler))
        .route("/set-config", post(set_config_handler))
        .route("/auth", post(auth_handler))
        .route("/ping", get(|| async { "pong" }))
        .route("/api/thumbs", post(upload_thumbs_handler))         // 👈 mudado
        .route("/api/thumbs/list", get(list_thumbs_handler))
        .merge(thumbs_route) // <- Aqui entra o ServeDir
        .merge(create_ws_router())
        .with_state(shared_state.clone())
        .layer(cors);



    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    if let Ok(ip) = local_ip() {
        println!("📱 Escaneie: http://{}:8080", ip);
    }

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await.unwrap();
}
