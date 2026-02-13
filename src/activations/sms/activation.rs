use super::providers::{create_provider, SmsProvider};
use super::types::*;
use crate::config::SmsConfig;
use async_stream::stream;
use futures::Stream;
use std::sync::Arc;

// Required for macro-generated code
use plexus_core::plexus;
use plexus_core::serde_helpers;

#[derive(Clone)]
pub struct Sms {
    provider: Arc<Box<dyn SmsProvider>>,
}

impl Sms {
    pub async fn new(config: SmsConfig) -> Result<Self, String> {
        let provider = create_provider(&config)?;

        Ok(Self {
            provider: Arc::new(provider),
        })
    }
}

#[plexus_macros::hub_methods(
    namespace = "sms",
    version = "1.0.0",
    description = "Send SMS messages via multiple providers (Twilio, SNS, Vonage, MessageBird)"
)]
impl Sms {
    #[plexus_macros::hub_method(
        description = "Send an SMS message",
        params(
            to = "Recipient phone number (E.164 format)",
            message = "Message content",
            from = "Sender phone number (optional, uses default if not provided)"
        )
    )]
    async fn send(
        &self,
        to: String,
        message: String,
        from: Option<String>,
    ) -> impl Stream<Item = SendSmsEvent> + Send + 'static {
        let provider = self.provider.clone();
        let params = SendSmsParams { to, message, from };

        stream! {
            match provider.send(params).await {
                Ok(event) => yield event,
                Err(e) => yield SendSmsEvent::Error {
                    message: e,
                    code: None,
                },
            }
        }
    }

    #[plexus_macros::hub_method(
        streaming,
        description = "Send multiple SMS messages with progress tracking",
        params(messages = "List of SMS messages to send")
    )]
    async fn send_batch(
        &self,
        messages: Vec<SendSmsParams>,
    ) -> impl Stream<Item = BatchSendEvent> + Send + 'static {
        let provider = self.provider.clone();
        let total = messages.len();

        stream! {
            let mut sent = 0;
            let mut failed = 0;

            for (index, sms) in messages.into_iter().enumerate() {
                let to = sms.to.clone();
                match provider.send(sms).await {
                    Ok(SendSmsEvent::Queued { message_id, .. }) |
                    Ok(SendSmsEvent::Sent { message_id, .. }) => {
                        sent += 1;
                        yield BatchSendEvent::SmsSent { index, message_id, to };
                    }
                    Ok(SendSmsEvent::Error { message, .. }) | Err(message) => {
                        failed += 1;
                        yield BatchSendEvent::SmsFailed { index, to, error: message };
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

    #[plexus_macros::hub_method(
        description = "Check delivery status of an SMS message",
        params(message_id = "Message ID returned from send")
    )]
    async fn check_status(
        &self,
        message_id: String,
    ) -> impl Stream<Item = StatusEvent> + Send + 'static {
        let provider = self.provider.clone();

        stream! {
            match provider.check_status(&message_id).await {
                Ok(status) => yield StatusEvent::Status { message_id, status },
                Err(e) => yield StatusEvent::Error { message: e },
            }
        }
    }
}
