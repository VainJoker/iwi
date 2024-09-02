use std::sync::Arc;

use super::Service;
use crate::{
    app::bootstrap::{
        constants::{MQ_SEND_EMAIL_QUEUE, MQ_SEND_EMAIL_TAG},
        AppState,
    },
    library::{error::AppResult, mailor::Email, mqer::Subscriber, Mqer},
};

#[derive(Clone)]
pub struct Server {
    pub mqer: Arc<Mqer>,
}

impl Service for Server {
    async fn init() -> Server {
        Server {
            mqer: Arc::new(Mqer::init()),
        }
    }

    async fn serve(&mut self, _app_state: Arc<AppState>) {
        match self.email_sender().await {
            Ok(()) => {}
            Err(e) => {
                tracing::error!("Error occurred while sending email: {}", e)
            }
        };
    }

    async fn shutdown(&self) {
        match self.mqer.graceful_shutdown() {
            Ok(()) => {}
            Err(e) => {
                tracing::error!("Error occurred while closing MQ: {}", e)
            }
        }
    }
}

impl Server {
    pub async fn email_sender(&self) -> AppResult<()> {
        tracing::debug!("email customer started");
        let func = |message: String| {
            let result = serde_json::from_str::<Email>(&message)
                .map_err(|e| {
                    tracing::error!("Failed to parse email from message: {}", e)
                })
                .and_then(|email| {
                    let res = email.sync_send_text().map_err(|e| {
                        tracing::error!("Failed to send email: {}", e)
                    });
                    tracing::debug!("received:{:#?}", email);
                    res
                });
            if result.is_err() {
                tracing::error!("Failed to send email")
            }
        };
        let delegate = Subscriber::new(func, self.mqer.clone());
        Ok(self
            .mqer
            .basic_receive(MQ_SEND_EMAIL_QUEUE, MQ_SEND_EMAIL_TAG, delegate)
            .await?)
    }
}
