mod handlers;
mod state;
mod tcp_server;
mod utils;
mod ws;

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[tokio::main]
async fn main() {
    frontend_lib::run().await;
}
