mod handlers;
mod state;
mod tcp_server;
mod utils;
mod ws;

use crate::handlers::auth::CodeResponse;
use crate::handlers::config::{set_config_handler, ConfigPayload};
use crate::handlers::thumbs::list_thumbs_handler;
use crate::state::{AppState, DbRequest};

use anyhow::Result;
use chrono::Utc;
use dirs::picture_dir;
use rand::{distributions::Alphanumeric, Rng};
use tauri_plugin_sql::{Migration, MigrationKind};
use tokio::{fs, sync::{mpsc, Mutex, RwLock}};

use std::{
    net::{TcpStream, SocketAddr},
    io::{Write, BufRead, BufReader}, 
    sync::Arc
};

#[tauri::command]
async fn thumbs_list(window: tauri::Window) {
    list_thumbs_handler(window).await;
}

#[tauri::command]
async fn set_config(
    state: tauri::State<'_, Arc<AppState>>,
    payload: ConfigPayload,
) -> Result<String, String> {
    set_config_handler(state.inner().clone(), payload).await
}

fn test_tcp() {
    match TcpStream::connect("127.0.0.1:7878") {
        Ok(mut stream) => {
            println!("✅ Conectado ao servidor!");

            // Exemplo de configuração (JSON string)
            let config = r#"{"mainDirecrtory":"C:\\Users\\BRUNO\\AppData\\Local\\Cube","user":"bruno"}"#;
            let command = format!("START {}\n", config);

            // Envia comando
            stream.write_all(command.as_bytes()).unwrap();
            println!("📤 Comando enviado: {}", command.trim());

            // Aguarda resposta
            let mut reader = BufReader::new(&stream);
            let mut response = String::new();
            reader.read_line(&mut response).unwrap();
            println!("📥 Resposta: {}", response.trim());
        }
        Err(e) => {
            println!("❌ Falha na conexão: {}", e);
        }
    }
}

#[tauri::command]
async fn get_qr_code(window: tauri::Window) -> Result<CodeResponse, String> {
    let now = Utc::now().to_rfc3339();
    let ip = local_ip_address::local_ip()
        .map_err(|e| e.to_string())?
        .to_string();
    let code = generate_code(6);

    Ok(CodeResponse {
        code,
        ip,
        expires_in: 60,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    //std::thread::spawn(|| {
    //    test_unsafe(); // FFI com COM via .NET
    //});
    
    test_tcp();
    let default_dir = prepare_upload_dir().await;
    let (db_tx, db_rx) = mpsc::channel::<DbRequest>(32);
    let app_state = AppState {
        upload_dir: Arc::new(RwLock::new(default_dir.clone())),
        ws_state: Arc::new(Mutex::new(Vec::new())),
        db_tx,
    };

    let shared_state = Arc::new(app_state);

    // Start HTTP/WebSocket
    let axum_state = shared_state.clone();
    tokio::spawn(async move {
        start_axum_server(axum_state).await;
    });

    //let tcp_server = start_tcp_server(shared_state.clone()).await;

    // Opcional: TCP server
    //tokio::spawn(tcp_server);

    let db_path = std::env::current_dir().unwrap().join("uploads.db");
    let db_url = format!("sqlite:{}", "uploads.db");

    let sql = include_str!("./migrations/create_tables.sql");
    println!("📄 SQL da migração:\n{}", db_url);

    let sql_plugin = tauri_plugin_sql::Builder::default()
        .add_migrations(
            &db_url,
            vec![Migration {
                version: 1,
                description: "create tables",
                sql: include_str!("./migrations/create_tables.sql"),
                kind: MigrationKind::Up,
            }],
        )
        .build();

    println!("✅ Migração registrada");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(sql_plugin)
        .invoke_handler(tauri::generate_handler![
            get_qr_code,
            set_config,
            thumbs_list
        ])
        .manage(shared_state)
        .run(tauri::generate_context!())
        .expect("Erro ao executar app Tauri");

    println!("✅ App Tauri executada com sucesso"); // só vai aparecer ao encerrar o app
}

async fn prepare_upload_dir() -> String {
    let dir = picture_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap().join("uploads"))
        .to_str()
        .unwrap()
        .to_string();

    fs::create_dir_all(&dir).await.unwrap();
    dir
}

async fn start_axum_server(state: Arc<AppState>) {
    use crate::handlers::{
        auth::auth_handler, thumbs::upload_thumbs_handler, upload_raw::upload_raw_handler,
    };
    use crate::ws::create_ws_router;
    use axum::{
        routing::{get, post},
        Router,
    };
    use local_ip_address::local_ip;
    use tower_http::{
        cors::{Any, CorsLayer},
        services::ServeDir,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let static_files = ServeDir::new(".thumbs").append_index_html_on_directories(false);
    let thumbs_route = Router::new().nest_service("/thumbs", static_files);

    let app = Router::new()
        .route("/ping", get(|| async { "pong" }))
        .route("/upload_raw", post(upload_raw_handler)) // stay
        .route("/auth", post(auth_handler)) // stay
        .route("/api/thumbs", post(upload_thumbs_handler)) // stay
        .merge(thumbs_route)
        .merge(create_ws_router())
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    if let Ok(ip) = local_ip() {
        println!("🌐 Servidor HTTP ativo em: http://{}:8080", ip);
    }

    println!("🌐 Servidor HTTP ativo em: http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

fn generate_code(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
