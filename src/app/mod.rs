pub mod api;
pub mod bootstrap;
pub mod entity;
pub mod service;

use std::sync::Arc;

use crate::app::bootstrap::AppState;

pub async fn serve() {
    let app_state = Arc::new(AppState::init().await);

    AppState::serve(app_state.clone()).await;

    api::Server::init(app_state.clone()).serve().await;

    app_state.services.shutdown().await;
}
