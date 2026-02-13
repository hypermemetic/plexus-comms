use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Request types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendPushParams {
    pub device_token: String,
    pub platform: Platform,
    pub title: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Ios,
    Android,
    Web,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendBatchParams {
    pub notifications: Vec<SendPushParams>,
}

// Response/Event types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SendPushEvent {
    Queued {
        message_id: String,
        platform: Platform,
    },
    Sent {
        message_id: String,
        platform: Platform,
        timestamp: i64,
    },
    Error {
        message: String,
        platform: Option<Platform>,
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
    NotificationSent {
        index: usize,
        message_id: String,
        platform: Platform,
    },
    NotificationFailed {
        index: usize,
        platform: Platform,
        error: String,
    },
    Complete {
        total_sent: usize,
        total_failed: usize,
    },
}
