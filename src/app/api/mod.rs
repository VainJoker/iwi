use std::sync::Arc;

use tokio::net::TcpListener;

use crate::{
    app::bootstrap::{shutdown_signal, AppState},
    library::cfg,
};

pub mod controller;
pub mod middleware;
pub mod route;

pub struct Server {
    pub host: &'static str,
    pub port: usize,
    pub app_state: Arc<AppState>,
}

impl Server {
    pub fn init(app_state: Arc<AppState>) -> Self {
        let config = cfg::config();
        let host = &config.app.host;
        let port = config.app.port;
        Self {
            host,
            port,
            app_state,
        }
    }

    pub async fn serve(self) {
        let app = route::init(self.app_state.clone());
        let listener =
            TcpListener::bind(format!("{}:{}", self.host, self.port))
                .await
                .unwrap_or_else(|e| {
                    panic!("💥 Failed to connect bind TcpListener: {e:?}")
                });

        tracing::info!(
            "✨ listening on {}",
            listener.local_addr().unwrap_or_else(|e| panic!(
                "💥 Failed to connect bind TcpListener: {e:?}"
            ))
        );

        // Run the server with graceful shutdown
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap_or_else(|e| panic!("💥 Failed to start API server: {e:?}"));
    }
}
