//! # Cube Server Main
//!
//! This is the entry point for the Cube server application.
//!
//! ## Features
//! - Initializes the SQLite database and creates required tables if they do not exist.
//! - Sets up the global application state, including upload directory, database connection, and WebSocket state.
//! - Configures all HTTP and WebSocket routes using Axum, including file upload, authentication, configuration, and thumbnail management.
//! - Serves static thumbnail files from the `.thumbs` directory.
//! - Enables permissive CORS for development and cross-origin requests.
//! - Prints the local IP address for easy access from other devices on the network.
//!
//! ## Endpoints
//! - `/upload_raw`: Upload RAW files.
//! - `/generate_code`: Generate authentication code.
//! - `/set-config`: Set or update upload directory.
//! - `/auth`: Authenticate and receive a session token.
//! - `/ping`: Health check endpoint.
//! - `/api/thumbs`: Upload thumbnails.
//! - `/api/thumbs/list`: List thumbnails.
//! - `/thumbs/*`: Serve static thumbnail files.
//! - WebSocket endpoint (see `ws` module).

mod state;
mod handlers;
mod utils;
mod ws;
mod tcp_server;

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
    // Set up default upload directory
    let default_dir = picture_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("uploads"))
        .to_str()
        .unwrap()
        .to_string();

    fs::create_dir_all(&default_dir).await.unwrap();

    // Initialize SQLite database and tables
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

    // Build global application state
    let state = AppState {
        upload_dir: Arc::new(RwLock::new(default_dir)),
        db: Arc::new(Mutex::new(conn)),
        ws_state: Arc::new(Mutex::new(Vec::new())),
    };
    
    // Serve static thumbnail files
    let static_files = ServeDir::new(".thumbs").append_index_html_on_directories(false);
    let thumbs_route = Router::new().nest_service("/thumbs", static_files);

    let shared_state = Arc::new(state);
    tokio::spawn(tcp_server::start_tcp_server(shared_state.clone()));

    // Enable permissive CORS
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    // Build Axum application with all routes
    let app = Router::new()
        .route("/upload_raw", post(upload_raw_handler))
        .route("/generate_code", get(generate_code_handler))
        .route("/set-config", post(set_config_handler))
        .route("/auth", post(auth_handler))
        .route("/ping", get(|| async { "pong" }))
        .route("/api/thumbs", post(upload_thumbs_handler))
        .route("/api/thumbs/list", get(list_thumbs_handler))
        .merge(thumbs_route)
        .merge(create_ws_router())
        .with_state(shared_state.clone())
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    if let Ok(ip) = local_ip() {
        println!("📱 Scan: http://{}:8080", ip);
    }

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await.unwrap();
}