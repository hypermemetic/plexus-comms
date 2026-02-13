use super::types::*;
use async_trait::async_trait;
use crate::config::PushConfig;

#[async_trait]
pub trait PushProvider: Send + Sync {
    async fn send(&self, notification: SendPushParams) -> Result<SendPushEvent, String>;
    fn supports_platform(&self, platform: &Platform) -> bool;
}

// APNs Provider
#[cfg(feature = "push-apns")]
pub struct ApnsProvider {}

#[cfg(feature = "push-apns")]
impl ApnsProvider {
    pub fn new(_config: &crate::config::ApnsConfig) -> Result<Self, String> {
        Ok(Self {})
    }
}

#[cfg(feature = "push-apns")]
#[async_trait]
impl PushProvider for ApnsProvider {
    async fn send(&self, params: SendPushParams) -> Result<SendPushEvent, String> {
        Ok(SendPushEvent::Queued {
            message_id: uuid::Uuid::new_v4().to_string(),
            platform: params.platform,
        })
    }

    fn supports_platform(&self, platform: &Platform) -> bool {
        matches!(platform, Platform::Ios)
    }
}

// FCM Provider
#[cfg(feature = "push-fcm")]
pub struct FcmProvider {
    client: reqwest::Client,
}

#[cfg(feature = "push-fcm")]
impl FcmProvider {
    pub fn new(_config: &crate::config::FcmConfig) -> Result<Self, String> {
        Ok(Self {
            client: reqwest::Client::new(),
        })
    }
}

#[cfg(feature = "push-fcm")]
#[async_trait]
impl PushProvider for FcmProvider {
    async fn send(&self, params: SendPushParams) -> Result<SendPushEvent, String> {
        Ok(SendPushEvent::Queued {
            message_id: uuid::Uuid::new_v4().to_string(),
            platform: params.platform,
        })
    }

    fn supports_platform(&self, platform: &Platform) -> bool {
        matches!(platform, Platform::Android)
    }
}

// Web Push Provider
#[cfg(feature = "push-webpush")]
pub struct WebPushProvider {}

#[cfg(feature = "push-webpush")]
impl WebPushProvider {
    pub fn new(_config: &crate::config::WebPushConfig) -> Result<Self, String> {
        Ok(Self {})
    }
}

#[cfg(feature = "push-webpush")]
#[async_trait]
impl PushProvider for WebPushProvider {
    async fn send(&self, params: SendPushParams) -> Result<SendPushEvent, String> {
        Ok(SendPushEvent::Queued {
            message_id: uuid::Uuid::new_v4().to_string(),
            platform: params.platform,
        })
    }

    fn supports_platform(&self, platform: &Platform) -> bool {
        matches!(platform, Platform::Web)
    }
}

// Router to select the right provider based on platform
pub struct PushRouter {
    #[cfg(feature = "push-apns")]
    apns: Option<ApnsProvider>,
    #[cfg(feature = "push-fcm")]
    fcm: Option<FcmProvider>,
    #[cfg(feature = "push-webpush")]
    web_push: Option<WebPushProvider>,
}

impl PushRouter {
    pub fn new(config: &PushConfig) -> Result<Self, String> {
        Ok(Self {
            #[cfg(feature = "push-apns")]
            apns: config
                .ios
                .as_ref()
                .map(ApnsProvider::new)
                .transpose()?,
            #[cfg(feature = "push-fcm")]
            fcm: config
                .android
                .as_ref()
                .map(FcmProvider::new)
                .transpose()?,
            #[cfg(feature = "push-webpush")]
            web_push: config
                .web
                .as_ref()
                .map(WebPushProvider::new)
                .transpose()?,
        })
    }

    pub async fn send(&self, notification: SendPushParams) -> Result<SendPushEvent, String> {
        match &notification.platform {
            Platform::Ios => {
                #[cfg(feature = "push-apns")]
                {
                    if let Some(provider) = &self.apns {
                        return provider.send(notification).await;
                    }
                }
                Err("iOS push notifications not configured".to_string())
            }
            Platform::Android => {
                #[cfg(feature = "push-fcm")]
                {
                    if let Some(provider) = &self.fcm {
                        return provider.send(notification).await;
                    }
                }
                Err("Android push notifications not configured".to_string())
            }
            Platform::Web => {
                #[cfg(feature = "push-webpush")]
                {
                    if let Some(provider) = &self.web_push {
                        return provider.send(notification).await;
                    }
                }
                Err("Web push notifications not configured".to_string())
            }
        }
    }
}
