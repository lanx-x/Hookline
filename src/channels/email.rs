use crate::notification::{level_style, Notification};
use async_trait::async_trait;
use lettre::message::header::ContentType;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use super::Channel;

pub struct EmailChannel {
    pub name: String,
    pub from: String,
    pub to: String,
    pub transport: AsyncSmtpTransport<Tokio1Executor>,
    pub format: String,
}

impl EmailChannel {
    pub fn new(
        name: String,
        smtp_host: &str,
        smtp_port: u16,
        username: &str,
        password: &str,
        from: &str,
        to: &str,
        format: &str,
    ) -> Result<Self, String> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(smtp_host)
            .map_err(|e| format!("failed to create smtp transport: {e}"))?
            .port(smtp_port)
            .credentials(lettre::transport::smtp::authentication::Credentials::new(
                username.to_string(),
                password.to_string(),
            ))
            .build();

        Ok(Self {
            name,
            from: from.to_string(),
            to: to.to_string(),
            transport,
            format: format.to_string(),
        })
    }
}

#[async_trait]
impl Channel for EmailChannel {
    fn name(&self) -> &str {
        &self.name
    }

    async fn send(&self, notification: &Notification) -> Result<(), String> {
        let to = notification.to.as_deref().unwrap_or(&self.to);
        let from = notification.from.as_deref().unwrap_or(&self.from);

        let from: lettre::message::Mailbox = from.parse().map_err(|e| format!("{e}"))?;
        let to: lettre::message::Mailbox = to.parse().map_err(|e| format!("{e}"))?;

        let style = level_style(&notification.level);

        let subject = format!("{} [{}] {}", style.emoji, style.label, notification.title);
        let body = format!(
            "<span style=\"display:inline-block;padding:2px 8px;border-radius:3px;background:{};color:#fff;font-weight:bold;font-size:12px\">{} {}</span>\n\n{}",
            style.hex_color, style.emoji, style.label, notification.message
        );

        let email = Message::builder()
            .from(from)
            .to(to)
            .subject(&subject)
            .header(ContentType::TEXT_HTML)
            .body(body)
            .map_err(|e| format!("failed to build email: {e}"))?;

        self.transport
            .send(email)
            .await
            .map_err(|e| format!("failed to send email: {e}"))?;

        log::info!("email sent to {}", notification.to.as_deref().unwrap_or(&self.to));
        Ok(())
    }
}
