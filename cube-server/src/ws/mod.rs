mod mobile;
mod ws; // já existente

pub use ws::*;
pub use mobile::*;

use axum::Router;
use crate::state::AppState;
use std::sync::Arc;

pub fn create_ws_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/ws", axum::routing::get(ws_handler))
        .route("/ws/mobile", axum::routing::get(ws_mobile_handler))
}
