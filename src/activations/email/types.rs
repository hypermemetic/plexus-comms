use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Re-export from submodules
pub use super::imap::EmailMessage;
pub use super::storage::{EmailAccount, SmtpAccountConfig, ImapAccountConfig};

// Request types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendEmailParams {
    pub to: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<String>>,
    pub subject: String,
    pub body: EmailBody,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EmailBody {
    Text { content: String },
    Html { content: String },
    Both { text: String, html: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Attachment {
    pub filename: String,
    pub content_type: String,
    pub data: String, // Base64-encoded
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendBatchParams {
    pub emails: Vec<SendEmailParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidateAddressParams {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetTemplateParams {
    pub template_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RenderTemplateParams {
    pub template_id: String,
    pub variables: serde_json::Value,
}

// Response/Event types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SendEmailEvent {
    Queued {
        message_id: String,
    },
    Sent {
        message_id: String,
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
    EmailSent {
        index: usize,
        message_id: String,
    },
    EmailFailed {
        index: usize,
        error: String,
    },
    Complete {
        total_sent: usize,
        total_failed: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValidateAddressEvent {
    Valid { email: String },
    Invalid { email: String, reason: String },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TemplateEvent {
    Template { template: EmailTemplate },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EmailTemplate {
    pub id: String,
    pub name: String,
    pub subject: String,
    pub body: String,
    pub variables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RenderTemplateEvent {
    Rendered {
        subject: String,
        body: String,
    },
    Error {
        message: String,
    },
}

// Account management events
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RegisterAccountEvent {
    Registered {
        account_name: String,
        has_smtp: bool,
        has_imap: bool,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ListAccountsEvent {
    Account {
        name: String,
        has_smtp: bool,
        has_imap: bool,
        created_at: i64,
    },
    Complete {
        total: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RemoveAccountEvent {
    Removed {
        account_name: String,
    },
    NotFound {
        account_name: String,
    },
    Error {
        message: String,
    },
}

// IMAP reading events
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReadInboxEvent {
    Message {
        message: EmailMessage,
    },
    Complete {
        total: usize,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SearchMessagesEvent {
    Message {
        message: EmailMessage,
    },
    Complete {
        total: usize,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MarkMessageEvent {
    Marked {
        uid: u32,
        status: String,
    },
    Error {
        message: String,
    },
}

// Parameters for account registration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RegisterAccountParams {
    pub name: String,
    pub smtp: Option<SmtpAccountConfig>,
    pub imap: Option<ImapAccountConfig>,
}
