use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Request types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendMessageParams {
    pub channel: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_ts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateChannelParams {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_private: Option<bool>,
}

// Response/Event types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SendMessageEvent {
    Sent {
        ts: String,
        channel: String,
    },
    Error {
        message: String,
        code: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChannelEvent {
    Created {
        channel_id: String,
        name: String,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SlackEvent {
    Message {
        ts: String,
        channel: String,
        user: String,
        text: String,
    },
    Reaction {
        reaction: String,
        user: String,
        item_ts: String,
    },
    Error {
        message: String,
    },
}
