use std::fmt::Debug;

use lettre::{
    message::header::ContentType,
    transport::smtp::{authentication::Credentials, response::Response},
    AsyncSmtpTransport, AsyncTransport, Message, SmtpTransport, Tokio1Executor,
    Transport,
};
use serde::{Deserialize, Serialize};

use crate::library::{
    cfg,
    cfg::MailConfig,
    error::{AppInnerError, InnerResult},
};

// TODO: masking the password in the log using macro
#[derive(Debug, Serialize, Deserialize)]
pub struct Email<'a> {
    pub to: &'a str,
    pub subject: &'a str,
    pub body: &'a str,
    pub config: MailConfig,
}

impl<'a> Email<'a> {
    pub fn new(to: &'a str, subject: &'a str, body: &'a str) -> Self {
        let config = cfg::config().mail.clone();
        Self {
            to,
            subject,
            body,
            config,
        }
    }

    pub fn sync_send_text(&self) -> InnerResult<Response> {
        let message = Message::builder()
            .from(self.config.username.parse().map_err(|e| {
                anyhow::anyhow!("Error occurred while sending message: {}", e)
            })?)
            .to(self.to.parse().map_err(|e| {
                anyhow::anyhow!("Error occurred while sending message: {}", e)
            })?)
            .subject(self.subject)
            .header(ContentType::TEXT_PLAIN) // ContentType::TEXT_HTML
            .body(self.body.to_string())
            .unwrap();
        let creds = Credentials::new(
            self.config.username.clone(),
            self.config.password.clone(),
        );
        let mailer = SmtpTransport::relay(&self.config.host)
            .map_err(|e| {
                tracing::error!("📧 Failed to send email: {e}");
                AppInnerError::EmailError(e)
            })?
            .credentials(creds)
            .build();
        Ok(mailer.send(&message)?)
    }

    pub async fn async_send_text(&self) -> InnerResult<Response> {
        let message = Message::builder()
            .from(self.config.username.parse().map_err(|e| {
                anyhow::anyhow!("Error occurred while sending message: {}", e)
            })?)
            .to(self.to.parse().map_err(|e| {
                anyhow::anyhow!("Error occurred while sending message: {}", e)
            })?)
            .subject(self.subject)
            .header(ContentType::TEXT_PLAIN) // ContentType::TEXT_HTML
            .body(self.body.to_string())
            .unwrap();
        let creds = Credentials::new(
            self.config.username.clone(),
            self.config.password.clone(),
        );

        let mailer =
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.host)
                .map_err(|e| {
                    tracing::error!("📧 Failed to send email: {e}");
                    AppInnerError::EmailError(e)
                })?
                .credentials(creds)
                .build();

        Ok(mailer.send(message).await?)
    }
}
