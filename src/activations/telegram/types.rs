use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Request types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendMessageParams {
    pub chat_id: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to_message_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub enum ParseMode {
    Markdown,
    Html,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendPhotoParams {
    pub chat_id: String,
    pub photo: String, // File ID or URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
}

// Response/Event types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SendMessageEvent {
    Sent {
        message_id: i64,
        chat_id: String,
    },
    Error {
        message: String,
        code: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UpdateEvent {
    Message {
        message_id: i64,
        chat_id: String,
        from_user: String,
        text: String,
        timestamp: i64,
    },
    CallbackQuery {
        query_id: String,
        from_user: String,
        data: String,
    },
    Error {
        message: String,
    },
}
