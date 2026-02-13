// Communication activations for Plexus RPC

// Email activation (multi-provider: SMTP, SendGrid, SES, Mailgun, Postmark)
pub mod email;

// SMS activation (multi-provider: Twilio, SNS, Vonage, MessageBird)
pub mod sms;

// Push notification activation (multi-platform: APNs, FCM, Web Push)
pub mod push;

// Messaging platform activations
pub mod telegram;
pub mod whatsapp;
pub mod slack;
pub mod discord;

// Re-exports
pub use email::Email;
pub use sms::Sms;
pub use push::Push;
pub use telegram::Telegram;
pub use whatsapp::Whatsapp;
pub use slack::Slack;
pub use discord::Discord;
