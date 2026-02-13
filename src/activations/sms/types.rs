use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Request types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendSmsParams {
    pub to: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendBatchParams {
    pub messages: Vec<SendSmsParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CheckStatusParams {
    pub message_id: String,
}

// Response/Event types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SendSmsEvent {
    Queued {
        message_id: String,
        to: String,
    },
    Sent {
        message_id: String,
        to: String,
        timestamp: i64,
    },
    Error {
        message: String,
        code: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BatchSendEvent {
    Progress {
        sent: usize,
        total: usize,
        percentage: f32,
    },
    SmsSent {
        index: usize,
        message_id: String,
        to: String,
    },
    SmsFailed {
        index: usize,
        to: String,
        error: String,
    },
    Complete {
        total_sent: usize,
        total_failed: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeliveryStatus {
    Queued,
    Sent,
    Delivered,
    Failed { reason: String },
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StatusEvent {
    Status {
        message_id: String,
        status: DeliveryStatus,
    },
    Error {
        message: String,
    },
}
