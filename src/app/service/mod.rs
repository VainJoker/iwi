use std::sync::Arc;

use crate::app::bootstrap::AppState;

pub mod jwt_service;
pub mod message_queue;

#[derive(Clone)]
pub struct Services {
    pub message_queue: message_queue::Server,
}

impl Services {
    pub async fn init() -> Services {
        Services {
            message_queue: message_queue::Server::init().await,
        }
    }

    pub async fn serve(&self, app_state: Arc<AppState>) {
        self.message_queue.clone().serve(app_state.clone()).await;
    }

    pub async fn shutdown(&self) {
        self.message_queue.shutdown().await;
    }
}

#[allow(async_fn_in_trait)]
pub trait Service {
    async fn init() -> Self;
    async fn serve(&mut self, app_state: Arc<AppState>);
    async fn shutdown(&self);
}
