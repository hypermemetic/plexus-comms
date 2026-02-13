use super::providers::PushRouter;
use super::types::*;
use crate::config::PushConfig;
use async_stream::stream;
use futures::Stream;
use std::sync::Arc;

// Required for macro-generated code
use plexus_core::plexus;
use plexus_core::serde_helpers;

#[derive(Clone)]
pub struct Push {
    router: Arc<PushRouter>,
}

impl Push {
    pub async fn new(config: PushConfig) -> Result<Self, String> {
        let router = PushRouter::new(&config)?;

        Ok(Self {
            router: Arc::new(router),
        })
    }
}

#[plexus_macros::hub_methods(
    namespace = "push",
    version = "1.0.0",
    description = "Send push notifications to iOS (APNs), Android (FCM), and Web"
)]
impl Push {
    #[plexus_macros::hub_method(
        description = "Send a push notification",
        params(
            device_token = "Device token",
            platform = "Target platform (ios, android, web)",
            title = "Notification title",
            body = "Notification body",
            data = "Custom data payload (optional)",
            badge = "Badge count for iOS (optional)",
            sound = "Sound file name (optional)"
        )
    )]
    async fn send(
        &self,
        device_token: String,
        platform: Platform,
        title: String,
        body: String,
        data: Option<std::collections::HashMap<String, String>>,
        badge: Option<i32>,
        sound: Option<String>,
    ) -> impl Stream<Item = SendPushEvent> + Send + 'static {
        let router = self.router.clone();
        let params = SendPushParams {
            device_token,
            platform,
            title,
            body,
            data,
            badge,
            sound,
        };

        stream! {
            match router.send(params).await {
                Ok(event) => yield event,
                Err(e) => yield SendPushEvent::Error {
                    message: e,
                    platform: None,
                    code: None,
                },
            }
        }
    }

    #[plexus_macros::hub_method(
        streaming,
        description = "Send multiple push notifications with progress tracking",
        params(notifications = "List of push notifications to send")
    )]
    async fn send_batch(
        &self,
        notifications: Vec<SendPushParams>,
    ) -> impl Stream<Item = BatchSendEvent> + Send + 'static {
        let router = self.router.clone();
        let total = notifications.len();

        stream! {
            let mut sent = 0;
            let mut failed = 0;

            for (index, notif) in notifications.into_iter().enumerate() {
                let platform = notif.platform.clone();
                match router.send(notif).await {
                    Ok(SendPushEvent::Queued { message_id, .. }) |
                    Ok(SendPushEvent::Sent { message_id, .. }) => {
                        sent += 1;
                        yield BatchSendEvent::NotificationSent { index, message_id, platform };
                    }
                    Ok(SendPushEvent::Error { message, .. }) | Err(message) => {
                        failed += 1;
                        yield BatchSendEvent::NotificationFailed { index, platform, error: message };
                    }
                }

                if (index + 1) % 10 == 0 || index + 1 == total {
                    yield BatchSendEvent::Progress {
                        sent,
                        total,
                        percentage: ((sent + failed) as f32 / total as f32) * 100.0,
                    };
                }
            }

            yield BatchSendEvent::Complete {
                total_sent: sent,
                total_failed: failed,
            };
        }
    }
}
