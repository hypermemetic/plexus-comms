use super::imap::{create_imap_provider, EmailMessage};
use super::smtp::{create_provider as create_smtp_provider, EmailProvider};
use super::storage::{EmailAccount, EmailStorage, EmailStorageConfig, ImapAccountConfig, SmtpAccountConfig};
use super::types::*;
use async_stream::stream;
use futures::Stream;
use std::collections::HashMap;
use std::sync::Arc;

// Required for macro-generated code
use plexus_core::plexus;
use plexus_core::serde_helpers;

#[derive(Clone)]
pub struct Email {
    storage: Arc<EmailStorage>,
    templates: Arc<tokio::sync::RwLock<HashMap<String, EmailTemplate>>>,
}

impl Email {
    pub async fn new() -> Result<Self, String> {
        let storage = EmailStorage::new(EmailStorageConfig::default()).await?;

        Ok(Self {
            storage: Arc::new(storage),
            templates: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }

    pub async fn with_config(config: EmailStorageConfig) -> Result<Self, String> {
        let storage = EmailStorage::new(config).await?;

        Ok(Self {
            storage: Arc::new(storage),
            templates: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        })
    }
}

#[plexus_macros::hub_methods(
    namespace = "email",
    version = "2.0.0",
    description = "Multi-account email with IMAP reading and SMTP sending"
)]
impl Email {
    // ==================== Account Management ====================

    #[plexus_macros::hub_method(
        description = "Register a new email account with SMTP and/or IMAP credentials",
        params(
            name = "Account name (typically the email address)",
            smtp = "SMTP configuration for sending (optional)",
            imap = "IMAP configuration for reading (optional)"
        )
    )]
    async fn register_account(
        &self,
        name: String,
        smtp: Option<SmtpAccountConfig>,
        imap: Option<ImapAccountConfig>,
    ) -> impl Stream<Item = RegisterAccountEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let now = chrono::Utc::now().timestamp();
            let account = EmailAccount {
                name: name.clone(),
                smtp: smtp.clone(),
                imap: imap.clone(),
                created_at: now,
                updated_at: now,
            };

            match storage.register_account(account).await {
                Ok(_) => yield RegisterAccountEvent::Registered {
                    account_name: name,
                    has_smtp: smtp.is_some(),
                    has_imap: imap.is_some(),
                },
                Err(e) => yield RegisterAccountEvent::Error { message: e },
            }
        }
    }

    #[plexus_macros::hub_method(
        streaming,
        description = "List all registered email accounts"
    )]
    async fn list_accounts(&self) -> impl Stream<Item = ListAccountsEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            match storage.list_accounts().await {
                Ok(accounts) => {
                    let total = accounts.len();
                    for account in accounts {
                        yield ListAccountsEvent::Account {
                            name: account.name,
                            has_smtp: account.smtp.is_some(),
                            has_imap: account.imap.is_some(),
                            created_at: account.created_at,
                        };
                    }
                    yield ListAccountsEvent::Complete { total };
                }
                Err(e) => {
                    // Return error as a complete event with 0 total
                    tracing::error!("Failed to list accounts: {}", e);
                    yield ListAccountsEvent::Complete { total: 0 };
                }
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Remove an email account",
        params(name = "Account name to remove")
    )]
    async fn remove_account(
        &self,
        name: String,
    ) -> impl Stream<Item = RemoveAccountEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            match storage.remove_account(&name).await {
                Ok(true) => yield RemoveAccountEvent::Removed { account_name: name },
                Ok(false) => yield RemoveAccountEvent::NotFound { account_name: name },
                Err(e) => yield RemoveAccountEvent::Error { message: e },
            }
        }
    }

    // ==================== SMTP Sending ====================

    #[plexus_macros::hub_method(
        description = "Send an email from a registered account",
        params(
            account = "Account name to send from",
            to = "Recipients",
            cc = "CC recipients (optional)",
            bcc = "BCC recipients (optional)",
            subject = "Email subject",
            body = "Email body (text, HTML, or both)",
            attachments = "File attachments (optional)",
            reply_to = "Reply-to address (optional)"
        )
    )]
    async fn send_from(
        &self,
        account: String,
        to: Vec<String>,
        cc: Option<Vec<String>>,
        bcc: Option<Vec<String>>,
        subject: String,
        body: EmailBody,
        attachments: Option<Vec<Attachment>>,
        reply_to: Option<String>,
    ) -> impl Stream<Item = SendEmailEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            // Get account
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield SendEmailEvent::Error {
                        message: format!("Account '{}' not found", account),
                        code: Some("ACCOUNT_NOT_FOUND".to_string()),
                    };
                    return;
                }
                Err(e) => {
                    yield SendEmailEvent::Error {
                        message: format!("Failed to load account: {}", e),
                        code: None,
                    };
                    return;
                }
            };

            // Check if SMTP is configured
            let smtp_config = match account_config.smtp {
                Some(config) => config,
                None => {
                    yield SendEmailEvent::Error {
                        message: format!("Account '{}' has no SMTP configuration", account),
                        code: Some("NO_SMTP_CONFIG".to_string()),
                    };
                    return;
                }
            };

            // Convert to EmailConfig format for provider
            let email_config = crate::config::EmailConfig {
                provider: crate::config::EmailProvider::Smtp,
                credentials: crate::config::EmailCredentials::Smtp {
                    smtp_host: smtp_config.host,
                    smtp_port: smtp_config.port,
                    smtp_username: smtp_config.username,
                    smtp_password: smtp_config.password,
                    smtp_from: smtp_config.from_email,
                },
            };

            // Create provider and send
            let provider = match create_smtp_provider(&email_config) {
                Ok(p) => p,
                Err(e) => {
                    yield SendEmailEvent::Error {
                        message: format!("Failed to create SMTP provider: {}", e),
                        code: None,
                    };
                    return;
                }
            };

            let params = SendEmailParams {
                to,
                cc,
                bcc,
                subject,
                body,
                attachments,
                reply_to,
            };

            match provider.send(params).await {
                Ok(event) => yield event,
                Err(e) => yield SendEmailEvent::Error {
                    message: e,
                    code: None,
                },
            }
        }
    }

    #[plexus_macros::hub_method(
        streaming,
        description = "Send multiple emails with progress tracking from a registered account",
        params(
            account = "Account name to send from",
            emails = "List of emails to send"
        )
    )]
    async fn send_batch_from(
        &self,
        account: String,
        emails: Vec<SendEmailParams>,
    ) -> impl Stream<Item = BatchSendEvent> + Send + 'static {
        let storage = self.storage.clone();
        let total = emails.len();

        stream! {
            // Get account
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield BatchSendEvent::Complete {
                        total_sent: 0,
                        total_failed: total,
                    };
                    return;
                }
                Err(_) => {
                    yield BatchSendEvent::Complete {
                        total_sent: 0,
                        total_failed: total,
                    };
                    return;
                }
            };

            // Check SMTP config
            let smtp_config = match account_config.smtp {
                Some(config) => config,
                None => {
                    yield BatchSendEvent::Complete {
                        total_sent: 0,
                        total_failed: total,
                    };
                    return;
                }
            };

            // Create provider
            let email_config = crate::config::EmailConfig {
                provider: crate::config::EmailProvider::Smtp,
                credentials: crate::config::EmailCredentials::Smtp {
                    smtp_host: smtp_config.host,
                    smtp_port: smtp_config.port,
                    smtp_username: smtp_config.username,
                    smtp_password: smtp_config.password,
                    smtp_from: smtp_config.from_email,
                },
            };

            let provider = match create_smtp_provider(&email_config) {
                Ok(p) => p,
                Err(_) => {
                    yield BatchSendEvent::Complete {
                        total_sent: 0,
                        total_failed: total,
                    };
                    return;
                }
            };

            let mut sent = 0;
            let mut failed = 0;

            for (index, email) in emails.into_iter().enumerate() {
                match provider.send(email).await {
                    Ok(SendEmailEvent::Sent { message_id, .. }) |
                    Ok(SendEmailEvent::Queued { message_id }) => {
                        sent += 1;
                        yield BatchSendEvent::EmailSent { index, message_id };
                    }
                    Ok(SendEmailEvent::Error { message, .. }) | Err(message) => {
                        failed += 1;
                        yield BatchSendEvent::EmailFailed { index, error: message };
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

    // ==================== IMAP Reading ====================

    #[plexus_macros::hub_method(
        streaming,
        description = "Read inbox messages from a registered account",
        params(
            account = "Account name to read from",
            limit = "Maximum number of messages to fetch (optional)"
        )
    )]
    async fn read_inbox(
        &self,
        account: String,
        limit: Option<u32>,
    ) -> impl Stream<Item = ReadInboxEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            // Get account
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield ReadInboxEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield ReadInboxEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            // Check IMAP config
            let imap_config = match account_config.imap {
                Some(config) => config,
                None => {
                    yield ReadInboxEvent::Error {
                        message: format!("Account '{}' has no IMAP configuration", account),
                    };
                    return;
                }
            };

            // Create IMAP provider
            #[cfg(feature = "email-imap")]
            {
                let provider = create_imap_provider(imap_config);

                match provider.fetch_messages(limit).await {
                    Ok(messages) => {
                        let total = messages.len();
                        for message in messages {
                            yield ReadInboxEvent::Message { message };
                        }
                        yield ReadInboxEvent::Complete { total };
                    }
                    Err(e) => {
                        yield ReadInboxEvent::Error { message: e };
                    }
                }
            }

            #[cfg(not(feature = "email-imap"))]
            {
                yield ReadInboxEvent::Error {
                    message: "IMAP support not enabled. Enable 'email-imap' feature.".to_string(),
                };
            }
        }
    }

    #[plexus_macros::hub_method(
        streaming,
        description = "Search messages in a registered account",
        params(
            account = "Account name to search in",
            query = "IMAP search query (e.g., 'FROM github', 'SUBJECT deploy')"
        )
    )]
    async fn search_messages(
        &self,
        account: String,
        query: String,
    ) -> impl Stream<Item = SearchMessagesEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            // Get account
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield SearchMessagesEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield SearchMessagesEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            // Check IMAP config
            let imap_config = match account_config.imap {
                Some(config) => config,
                None => {
                    yield SearchMessagesEvent::Error {
                        message: format!("Account '{}' has no IMAP configuration", account),
                    };
                    return;
                }
            };

            // Create IMAP provider
            #[cfg(feature = "email-imap")]
            {
                let provider = create_imap_provider(imap_config);

                match provider.search_messages(&query).await {
                    Ok(messages) => {
                        let total = messages.len();
                        for message in messages {
                            yield SearchMessagesEvent::Message { message };
                        }
                        yield SearchMessagesEvent::Complete { total };
                    }
                    Err(e) => {
                        yield SearchMessagesEvent::Error { message: e };
                    }
                }
            }

            #[cfg(not(feature = "email-imap"))]
            {
                yield SearchMessagesEvent::Error {
                    message: "IMAP support not enabled. Enable 'email-imap' feature.".to_string(),
                };
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Mark a message as read",
        params(
            account = "Account name",
            uid = "Message UID"
        )
    )]
    async fn mark_read(
        &self,
        account: String,
        uid: u32,
    ) -> impl Stream<Item = MarkMessageEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield MarkMessageEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield MarkMessageEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let imap_config = match account_config.imap {
                Some(config) => config,
                None => {
                    yield MarkMessageEvent::Error {
                        message: format!("Account '{}' has no IMAP configuration", account),
                    };
                    return;
                }
            };

            #[cfg(feature = "email-imap")]
            {
                let provider = create_imap_provider(imap_config);

                match provider.mark_seen(uid).await {
                    Ok(_) => yield MarkMessageEvent::Marked {
                        uid,
                        status: "read".to_string(),
                    },
                    Err(e) => yield MarkMessageEvent::Error { message: e },
                }
            }

            #[cfg(not(feature = "email-imap"))]
            {
                yield MarkMessageEvent::Error {
                    message: "IMAP support not enabled.".to_string(),
                };
            }
        }
    }

    #[plexus_macros::hub_method(
        description = "Mark a message as unread",
        params(
            account = "Account name",
            uid = "Message UID"
        )
    )]
    async fn mark_unread(
        &self,
        account: String,
        uid: u32,
    ) -> impl Stream<Item = MarkMessageEvent> + Send + 'static {
        let storage = self.storage.clone();

        stream! {
            let account_config = match storage.get_account(&account).await {
                Ok(Some(acc)) => acc,
                Ok(None) => {
                    yield MarkMessageEvent::Error {
                        message: format!("Account '{}' not found", account),
                    };
                    return;
                }
                Err(e) => {
                    yield MarkMessageEvent::Error {
                        message: format!("Failed to load account: {}", e),
                    };
                    return;
                }
            };

            let imap_config = match account_config.imap {
                Some(config) => config,
                None => {
                    yield MarkMessageEvent::Error {
                        message: format!("Account '{}' has no IMAP configuration", account),
                    };
                    return;
                }
            };

            #[cfg(feature = "email-imap")]
            {
                let provider = create_imap_provider(imap_config);

                match provider.mark_unseen(uid).await {
                    Ok(_) => yield MarkMessageEvent::Marked {
                        uid,
                        status: "unread".to_string(),
                    },
                    Err(e) => yield MarkMessageEvent::Error { message: e },
                }
            }

            #[cfg(not(feature = "email-imap"))]
            {
                yield MarkMessageEvent::Error {
                    message: "IMAP support not enabled.".to_string(),
                };
            }
        }
    }

    // ==================== Template Management (kept from original) ====================

    #[plexus_macros::hub_method(
        description = "Validate an email address",
        params(email = "Email address to validate")
    )]
    async fn validate_address(
        &self,
        email: String,
    ) -> impl Stream<Item = ValidateAddressEvent> + Send + 'static {
        stream! {
            // Basic validation
            if email.contains('@') && email.contains('.') {
                yield ValidateAddressEvent::Valid { email };
            } else {
                yield ValidateAddressEvent::Invalid {
                    email,
                    reason: "Invalid email format".to_string(),
                };
            }
        }
    }
}
