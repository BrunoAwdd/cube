mod ws; // já existente

pub use ws::*;

use crate::state::AppState;
use axum::Router;
use std::sync::Arc;

pub fn create_ws_router() -> Router<Arc<AppState>> {
    Router::new().route("/ws", axum::routing::get(ws_handler))
}
