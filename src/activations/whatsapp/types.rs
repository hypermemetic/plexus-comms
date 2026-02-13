use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Request types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendMessageParams {
    pub to: String, // Phone number in E.164 format
    pub message: MessageContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    Text { body: String },
    Template { name: String, language: String },
    Media { url: String, caption: Option<String> },
}

// Response/Event types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SendMessageEvent {
    Sent {
        message_id: String,
        to: String,
    },
    Error {
        message: String,
        code: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebhookEvent {
    Message {
        message_id: String,
        from: String,
        timestamp: i64,
        content: MessageContent,
    },
    Status {
        message_id: String,
        status: DeliveryStatus,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatus {
    Sent,
    Delivered,
    Read,
    Failed,
}
