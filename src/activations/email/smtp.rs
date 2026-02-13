use super::types::*;
use async_trait::async_trait;
use crate::config::{EmailConfig, EmailCredentials, EmailProvider as EmailProviderType};

#[async_trait]
pub trait EmailProvider: Send + Sync {
    async fn send(&self, email: SendEmailParams) -> Result<SendEmailEvent, String>;
    async fn validate_address(&self, email: &str) -> Result<bool, String>;
}

// Factory function to create the appropriate provider
pub fn create_provider(config: &EmailConfig) -> Result<Box<dyn EmailProvider>, String> {
    match config.provider {
        EmailProviderType::Smtp => {
            #[cfg(feature = "email-smtp")]
            {
                Ok(Box::new(SmtpProvider::new(config)?))
            }
            #[cfg(not(feature = "email-smtp"))]
            {
                Err("SMTP provider not enabled. Enable 'email-smtp' feature.".to_string())
            }
        }
        EmailProviderType::SendGrid => {
            #[cfg(feature = "email-sendgrid")]
            {
                Ok(Box::new(SendGridProvider::new(config)?))
            }
            #[cfg(not(feature = "email-sendgrid"))]
            {
                Err("SendGrid provider not enabled. Enable 'email-sendgrid' feature.".to_string())
            }
        }
        EmailProviderType::Ses => {
            #[cfg(feature = "email-ses")]
            {
                Ok(Box::new(SesProvider::new(config)?))
            }
            #[cfg(not(feature = "email-ses"))]
            {
                Err("SES provider not enabled. Enable 'email-ses' feature.".to_string())
            }
        }
        EmailProviderType::Mailgun => {
            #[cfg(feature = "email-mailgun")]
            {
                Ok(Box::new(MailgunProvider::new(config)?))
            }
            #[cfg(not(feature = "email-mailgun"))]
            {
                Err("Mailgun provider not enabled. Enable 'email-mailgun' feature.".to_string())
            }
        }
        EmailProviderType::Postmark => {
            #[cfg(feature = "email-postmark")]
            {
                Ok(Box::new(PostmarkProvider::new(config)?))
            }
            #[cfg(not(feature = "email-postmark"))]
            {
                Err("Postmark provider not enabled. Enable 'email-postmark' feature.".to_string())
            }
        }
    }
}

// SMTP Provider implementation
#[cfg(feature = "email-smtp")]
pub struct SmtpProvider {
    config: EmailCredentials,
}

#[cfg(feature = "email-smtp")]
impl SmtpProvider {
    pub fn new(config: &EmailConfig) -> Result<Self, String> {
        Ok(Self {
            config: config.credentials.clone(),
        })
    }
}

#[cfg(feature = "email-smtp")]
#[async_trait]
impl EmailProvider for SmtpProvider {
    async fn send(&self, params: SendEmailParams) -> Result<SendEmailEvent, String> {
        use lettre::message::header::ContentType;
        use lettre::transport::smtp::authentication::Credentials;
        use lettre::{Message, SmtpTransport, Transport};

        let EmailCredentials::Smtp {
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            smtp_from,
        } = &self.config
        else {
            return Err("Invalid SMTP configuration".to_string());
        };

        // Build message
        let mut email_builder = Message::builder()
            .from(smtp_from.parse().map_err(|e| format!("Invalid from address: {}", e))?)
            .subject(&params.subject);

        for to in &params.to {
            email_builder = email_builder
                .to(to.parse().map_err(|e| format!("Invalid to address: {}", e))?);
        }

        if let Some(cc) = &params.cc {
            for addr in cc {
                email_builder = email_builder
                    .cc(addr.parse().map_err(|e| format!("Invalid cc address: {}", e))?);
            }
        }

        if let Some(reply_to) = &params.reply_to {
            email_builder = email_builder
                .reply_to(reply_to.parse().map_err(|e| format!("Invalid reply-to address: {}", e))?);
        }

        let email = match &params.body {
            EmailBody::Text { content } => email_builder.body(content.clone()),
            EmailBody::Html { content } => email_builder
                .header(ContentType::TEXT_HTML)
                .body(content.clone()),
            EmailBody::Both { text, html: _ } => {
                // For simplicity, just use text for now
                // Full implementation would use multipart
                email_builder.body(text.clone())
            }
        }
        .map_err(|e| format!("Failed to build email: {}", e))?;

        // Send via SMTP
        let creds = Credentials::new(smtp_username.clone(), smtp_password.clone());
        let mailer = SmtpTransport::relay(smtp_host)
            .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
            .port(*smtp_port)
            .credentials(creds)
            .build();

        match mailer.send(&email) {
            Ok(response) => {
                let message_id = format!("{:?}", response);
                Ok(SendEmailEvent::Sent {
                    message_id,
                    timestamp: chrono::Utc::now().timestamp(),
                })
            }
            Err(e) => Ok(SendEmailEvent::Error {
                message: format!("SMTP error: {}", e),
                code: None,
            }),
        }
    }

    async fn validate_address(&self, email: &str) -> Result<bool, String> {
        // Basic validation
        Ok(email.contains('@') && email.contains('.'))
    }
}

// SendGrid Provider implementation
#[cfg(feature = "email-sendgrid")]
pub struct SendGridProvider {
    api_key: String,
    from_email: String,
    from_name: Option<String>,
    client: reqwest::Client,
}

#[cfg(feature = "email-sendgrid")]
impl SendGridProvider {
    pub fn new(config: &EmailConfig) -> Result<Self, String> {
        let EmailCredentials::ApiKey {
            api_key,
            from_email,
            from_name,
        } = &config.credentials
        else {
            return Err("Invalid SendGrid configuration".to_string());
        };

        Ok(Self {
            api_key: api_key.clone(),
            from_email: from_email.clone(),
            from_name: from_name.clone(),
            client: reqwest::Client::new(),
        })
    }
}

#[cfg(feature = "email-sendgrid")]
#[async_trait]
impl EmailProvider for SendGridProvider {
    async fn send(&self, params: SendEmailParams) -> Result<SendEmailEvent, String> {
        // SendGrid API implementation would go here
        // For now, return a placeholder
        Ok(SendEmailEvent::Queued {
            message_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    async fn validate_address(&self, email: &str) -> Result<bool, String> {
        Ok(email.contains('@') && email.contains('.'))
    }
}

// SES Provider implementation
#[cfg(feature = "email-ses")]
pub struct SesProvider {
    // AWS SDK client would go here
}

#[cfg(feature = "email-ses")]
impl SesProvider {
    pub fn new(_config: &EmailConfig) -> Result<Self, String> {
        Ok(Self {})
    }
}

#[cfg(feature = "email-ses")]
#[async_trait]
impl EmailProvider for SesProvider {
    async fn send(&self, _params: SendEmailParams) -> Result<SendEmailEvent, String> {
        // AWS SES implementation would go here
        Ok(SendEmailEvent::Queued {
            message_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    async fn validate_address(&self, email: &str) -> Result<bool, String> {
        Ok(email.contains('@') && email.contains('.'))
    }
}

// Mailgun Provider implementation
#[cfg(feature = "email-mailgun")]
pub struct MailgunProvider {
    api_key: String,
    domain: String,
    client: reqwest::Client,
}

#[cfg(feature = "email-mailgun")]
impl MailgunProvider {
    pub fn new(config: &EmailConfig) -> Result<Self, String> {
        let EmailCredentials::ApiKey { api_key, .. } = &config.credentials else {
            return Err("Invalid Mailgun configuration".to_string());
        };

        Ok(Self {
            api_key: api_key.clone(),
            domain: "example.com".to_string(), // Should come from config
            client: reqwest::Client::new(),
        })
    }
}

#[cfg(feature = "email-mailgun")]
#[async_trait]
impl EmailProvider for MailgunProvider {
    async fn send(&self, _params: SendEmailParams) -> Result<SendEmailEvent, String> {
        Ok(SendEmailEvent::Queued {
            message_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    async fn validate_address(&self, email: &str) -> Result<bool, String> {
        Ok(email.contains('@') && email.contains('.'))
    }
}

// Postmark Provider implementation
#[cfg(feature = "email-postmark")]
pub struct PostmarkProvider {
    api_key: String,
    client: reqwest::Client,
}

#[cfg(feature = "email-postmark")]
impl PostmarkProvider {
    pub fn new(config: &EmailConfig) -> Result<Self, String> {
        let EmailCredentials::ApiKey { api_key, .. } = &config.credentials else {
            return Err("Invalid Postmark configuration".to_string());
        };

        Ok(Self {
            api_key: api_key.clone(),
            client: reqwest::Client::new(),
        })
    }
}

#[cfg(feature = "email-postmark")]
#[async_trait]
impl EmailProvider for PostmarkProvider {
    async fn send(&self, _params: SendEmailParams) -> Result<SendEmailEvent, String> {
        Ok(SendEmailEvent::Queued {
            message_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    async fn validate_address(&self, email: &str) -> Result<bool, String> {
        Ok(email.contains('@') && email.contains('.'))
    }
}
