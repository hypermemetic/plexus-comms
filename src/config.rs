use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommsConfig {
    pub email: Option<EmailConfig>,
    pub sms: Option<SmsConfig>,
    pub push: Option<PushConfig>,
    pub telegram: Option<TelegramConfig>,
    pub whatsapp: Option<WhatsappConfig>,
    pub slack: Option<SlackConfig>,
    pub discord: Option<DiscordConfig>,
}

impl Default for CommsConfig {
    fn default() -> Self {
        Self {
            email: Some(EmailConfig::default()),
            sms: None,
            push: None,
            telegram: None,
            whatsapp: None,
            slack: None,
            discord: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub provider: EmailProvider,
    #[serde(flatten)]
    pub credentials: EmailCredentials,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum EmailProvider {
    Smtp,
    SendGrid,
    Ses,
    Mailgun,
    Postmark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmailCredentials {
    Smtp {
        smtp_host: String,
        smtp_port: u16,
        smtp_username: String,
        smtp_password: String,
        smtp_from: String,
    },
    ApiKey {
        api_key: String,
        from_email: String,
        from_name: Option<String>,
    },
    Aws {
        region: String,
        from_email: String,
    },
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            provider: EmailProvider::Smtp,
            credentials: EmailCredentials::Smtp {
                smtp_host: "localhost".to_string(),
                smtp_port: 25,
                smtp_username: "".to_string(),
                smtp_password: "".to_string(),
                smtp_from: "noreply@localhost".to_string(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsConfig {
    pub provider: SmsProvider,
    #[serde(flatten)]
    pub credentials: SmsCredentials,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum SmsProvider {
    Twilio,
    Sns,
    Vonage,
    MessageBird,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SmsCredentials {
    Twilio {
        account_sid: String,
        auth_token: String,
        from_number: String,
    },
    Aws {
        region: String,
    },
    ApiKey {
        api_key: String,
        api_secret: Option<String>,
        from_number: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushConfig {
    pub ios: Option<ApnsConfig>,
    pub android: Option<FcmConfig>,
    pub web: Option<WebPushConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApnsConfig {
    pub key_file: PathBuf,
    pub key_id: String,
    pub team_id: String,
    pub environment: ApnsEnvironment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApnsEnvironment {
    Production,
    Sandbox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcmConfig {
    pub service_account_key: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPushConfig {
    pub vapid_private_key: String,
    pub vapid_public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub mode: TelegramMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TelegramMode {
    Polling,
    Webhook { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsappConfig {
    pub mode: WhatsappMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum WhatsappMode {
    BusinessApi {
        phone_number_id: String,
        access_token: String,
        webhook_verify_token: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub bot_token: String,
    pub app_token: Option<String>,
    pub mode: SlackMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SlackMode {
    Socket,
    Webhook { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub bot_token: String,
}
