use super::types::*;
use async_trait::async_trait;
use crate::config::{SmsConfig, SmsCredentials, SmsProvider as SmsProviderType};

#[async_trait]
pub trait SmsProvider: Send + Sync {
    async fn send(&self, sms: SendSmsParams) -> Result<SendSmsEvent, String>;
    async fn check_status(&self, message_id: &str) -> Result<DeliveryStatus, String>;
}

// Factory function
pub fn create_provider(config: &SmsConfig) -> Result<Box<dyn SmsProvider>, String> {
    match config.provider {
        SmsProviderType::Twilio => {
            #[cfg(feature = "sms-twilio")]
            {
                Ok(Box::new(TwilioProvider::new(config)?))
            }
            #[cfg(not(feature = "sms-twilio"))]
            {
                Err("Twilio provider not enabled".to_string())
            }
        }
        SmsProviderType::Sns => {
            #[cfg(feature = "sms-sns")]
            {
                Ok(Box::new(SnsProvider::new(config)?))
            }
            #[cfg(not(feature = "sms-sns"))]
            {
                Err("SNS provider not enabled".to_string())
            }
        }
        SmsProviderType::Vonage => {
            #[cfg(feature = "sms-vonage")]
            {
                Ok(Box::new(VonageProvider::new(config)?))
            }
            #[cfg(not(feature = "sms-vonage"))]
            {
                Err("Vonage provider not enabled".to_string())
            }
        }
        SmsProviderType::MessageBird => {
            #[cfg(feature = "sms-messagebird")]
            {
                Ok(Box::new(MessageBirdProvider::new(config)?))
            }
            #[cfg(not(feature = "sms-messagebird"))]
            {
                Err("MessageBird provider not enabled".to_string())
            }
        }
    }
}

// Twilio Provider
#[cfg(feature = "sms-twilio")]
pub struct TwilioProvider {
    account_sid: String,
    auth_token: String,
    from_number: String,
    client: reqwest::Client,
}

#[cfg(feature = "sms-twilio")]
impl TwilioProvider {
    pub fn new(config: &SmsConfig) -> Result<Self, String> {
        let SmsCredentials::Twilio {
            account_sid,
            auth_token,
            from_number,
        } = &config.credentials
        else {
            return Err("Invalid Twilio configuration".to_string());
        };

        Ok(Self {
            account_sid: account_sid.clone(),
            auth_token: auth_token.clone(),
            from_number: from_number.clone(),
            client: reqwest::Client::new(),
        })
    }
}

#[cfg(feature = "sms-twilio")]
#[async_trait]
impl SmsProvider for TwilioProvider {
    async fn send(&self, params: SendSmsParams) -> Result<SendSmsEvent, String> {
        // Twilio API implementation would go here
        Ok(SendSmsEvent::Queued {
            message_id: uuid::Uuid::new_v4().to_string(),
            to: params.to,
        })
    }

    async fn check_status(&self, _message_id: &str) -> Result<DeliveryStatus, String> {
        Ok(DeliveryStatus::Unknown)
    }
}

// SNS Provider
#[cfg(feature = "sms-sns")]
pub struct SnsProvider {}

#[cfg(feature = "sms-sns")]
impl SnsProvider {
    pub fn new(_config: &SmsConfig) -> Result<Self, String> {
        Ok(Self {})
    }
}

#[cfg(feature = "sms-sns")]
#[async_trait]
impl SmsProvider for SnsProvider {
    async fn send(&self, params: SendSmsParams) -> Result<SendSmsEvent, String> {
        Ok(SendSmsEvent::Queued {
            message_id: uuid::Uuid::new_v4().to_string(),
            to: params.to,
        })
    }

    async fn check_status(&self, _message_id: &str) -> Result<DeliveryStatus, String> {
        Ok(DeliveryStatus::Unknown)
    }
}

// Vonage Provider
#[cfg(feature = "sms-vonage")]
pub struct VonageProvider {
    api_key: String,
    api_secret: String,
    from_number: String,
    client: reqwest::Client,
}

#[cfg(feature = "sms-vonage")]
impl VonageProvider {
    pub fn new(config: &SmsConfig) -> Result<Self, String> {
        let SmsCredentials::ApiKey {
            api_key,
            api_secret,
            from_number,
        } = &config.credentials
        else {
            return Err("Invalid Vonage configuration".to_string());
        };

        Ok(Self {
            api_key: api_key.clone(),
            api_secret: api_secret.clone().unwrap_or_default(),
            from_number: from_number.clone(),
            client: reqwest::Client::new(),
        })
    }
}

#[cfg(feature = "sms-vonage")]
#[async_trait]
impl SmsProvider for VonageProvider {
    async fn send(&self, params: SendSmsParams) -> Result<SendSmsEvent, String> {
        Ok(SendSmsEvent::Queued {
            message_id: uuid::Uuid::new_v4().to_string(),
            to: params.to,
        })
    }

    async fn check_status(&self, _message_id: &str) -> Result<DeliveryStatus, String> {
        Ok(DeliveryStatus::Unknown)
    }
}

// MessageBird Provider
#[cfg(feature = "sms-messagebird")]
pub struct MessageBirdProvider {
    api_key: String,
    from_number: String,
    client: reqwest::Client,
}

#[cfg(feature = "sms-messagebird")]
impl MessageBirdProvider {
    pub fn new(config: &SmsConfig) -> Result<Self, String> {
        let SmsCredentials::ApiKey {
            api_key,
            from_number,
            ..
        } = &config.credentials
        else {
            return Err("Invalid MessageBird configuration".to_string());
        };

        Ok(Self {
            api_key: api_key.clone(),
            from_number: from_number.clone(),
            client: reqwest::Client::new(),
        })
    }
}

#[cfg(feature = "sms-messagebird")]
#[async_trait]
impl SmsProvider for MessageBirdProvider {
    async fn send(&self, params: SendSmsParams) -> Result<SendSmsEvent, String> {
        Ok(SendSmsEvent::Queued {
            message_id: uuid::Uuid::new_v4().to_string(),
            to: params.to,
        })
    }

    async fn check_status(&self, _message_id: &str) -> Result<DeliveryStatus, String> {
        Ok(DeliveryStatus::Unknown)
    }
}
