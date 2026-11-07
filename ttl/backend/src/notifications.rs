use std::{fmt, time::Duration};

use anyhow::{Context, Result};

use crate::configuration::Configuration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationChannel {
    Admin,
    Access,
}

impl fmt::Display for NotificationChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationChannel::Admin => write!(f, "admin"),
            NotificationChannel::Access => write!(f, "access"),
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationKind {
    DoorEvent,
    Registration,
    Error,
    Custom,
}

impl fmt::Display for NotificationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationKind::DoorEvent => write!(f, "door_event"),
            NotificationKind::Registration => write!(f, "registration"),
            NotificationKind::Error => write!(f, "error"),
            NotificationKind::Custom => write!(f, "custom"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct DiscordNotifier {
    admin_webhook_url: Option<String>,
    access_webhook_url: Option<String>,
    timeout: Duration,
}

#[allow(dead_code)]
impl DiscordNotifier {
    pub fn from_configuration(configuration: &Configuration) -> Self {
        Self {
            admin_webhook_url: configuration
                .discord_admin_webhook_url
                .clone()
                .filter(|value| !value.trim().is_empty()),
            access_webhook_url: configuration
                .discord_access_webhook_url
                .clone()
                .filter(|value| !value.trim().is_empty()),
            timeout: Duration::from_secs(10),
        }
    }

    pub fn notify(&self, channel: NotificationChannel, title: &str, message: &str) -> Result<()> {
        self.notify_with_kind(
            channel,
            NotificationKind::Custom,
            title,
            message,
        )
    }

    pub fn notify_with_kind(
        &self,
        channel: NotificationChannel,
        kind: NotificationKind,
        title: &str,
        message: &str,
    ) -> Result<()> {
        let webhook_url = self
            .webhook_url_for(channel)
            .with_context(|| format!("no Discord webhook configured for {channel} channel"))?;

        let payload = serde_json::json!({
            "content": format!("**{}**\n{}", title, message),
            "username": "TTL Bot",
            "embeds": [{
                "title": title,
                "description": message,
                "color": match kind {
                    NotificationKind::DoorEvent => 0x2ECC71,
                    NotificationKind::Registration => 0x3498DB,
                    NotificationKind::Error => 0xE74C3C,
                    NotificationKind::Custom => 0x95A5A6,
                },
                "footer": { "text": format!("event_type={}", kind) }
            }]
        });

        let agent = ureq::AgentBuilder::new()
            .timeout_connect(self.timeout)
            .timeout_read(self.timeout)
            .build();

        let response = agent
            .post(&webhook_url)
            .send_json(&payload)
            .with_context(|| format!("failed to deliver Discord notification to {channel}"))?;

        if !(200..=299).contains(&response.status()) {
            anyhow::bail!(
                "Discord webhook returned HTTP {} while sending notification to {}",
                response.status(),
                channel
            );
        }

        Ok(())
    }

    pub fn notify_admin(&self, title: &str, message: &str) -> Result<()> {
        self.notify_with_kind(NotificationChannel::Admin, NotificationKind::Custom, title, message)
    }

    pub fn notify_access(&self, title: &str, message: &str) -> Result<()> {
        self.notify_with_kind(NotificationChannel::Access, NotificationKind::Custom, title, message)
    }

    pub fn notify_door_event(&self, member_name: &str, action: &str, timestamp: &str) -> Result<()> {
        self.notify_with_kind(
            NotificationChannel::Admin,
            NotificationKind::DoorEvent,
            "Door event",
            &format!("{} triggered a door event ({}) at {}.", member_name, action, timestamp),
        )
    }

    pub fn notify_registration(
        &self,
        member_id: &str,
        member_name: &str,
        email: &str,
    ) -> Result<()> {
        self.notify_with_kind(
            NotificationChannel::Access,
            NotificationKind::Registration,
            "Ny registrering i TTL",
            &format!(
                "{} ({}) registrerte seg og må aktiveres. ID: {}.",
                member_name, email, member_id
            ),
        )
    }

    pub fn notify_error(&self, context: &str, message: &str) -> Result<()> {
        self.notify_with_kind(
            NotificationChannel::Admin,
            NotificationKind::Error,
            "TTL service error",
            &format!("{}: {}", context, message),
        )
    }

    fn webhook_url_for(&self, channel: NotificationChannel) -> Option<String> {
        match channel {
            NotificationChannel::Admin => self.admin_webhook_url.clone(),
            NotificationChannel::Access => self.access_webhook_url.clone(),
        }
    }
}
