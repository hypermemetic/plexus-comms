use super::storage::ImapAccountConfig;
use async_trait::async_trait;
use tokio_util::compat::TokioAsyncReadCompatExt;
use futures::StreamExt;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct EmailMessage {
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    pub date: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub is_seen: bool,
}

#[async_trait]
pub trait ImapProvider: Send + Sync {
    async fn fetch_messages(&self, limit: Option<u32>) -> Result<Vec<EmailMessage>, String>;
    async fn search_messages(&self, query: &str) -> Result<Vec<EmailMessage>, String>;
    async fn mark_seen(&self, uid: u32) -> Result<(), String>;
    async fn mark_unseen(&self, uid: u32) -> Result<(), String>;
}

#[cfg(feature = "email-imap")]
pub struct AsyncImapProvider {
    config: ImapAccountConfig,
}

#[cfg(feature = "email-imap")]
impl AsyncImapProvider {
    pub fn new(config: ImapAccountConfig) -> Self {
        Self { config }
    }

    async fn connect(&self) -> Result<async_imap::Session<async_native_tls::TlsStream<tokio_util::compat::Compat<tokio::net::TcpStream>>>, String> {
        let tcp_stream = tokio::net::TcpStream::connect((self.config.host.as_str(), self.config.port))
            .await
            .map_err(|e| format!("Failed to connect to TCP: {}", e))?;

        // Convert tokio AsyncRead/AsyncWrite to futures AsyncRead/AsyncWrite
        let tcp_stream_compat = tcp_stream.compat();

        let tls = async_native_tls::TlsConnector::new();
        let tls_stream = tls
            .connect(&self.config.host, tcp_stream_compat)
            .await
            .map_err(|e| format!("Failed to establish TLS: {}", e))?;

        let client = async_imap::Client::new(tls_stream);

        let session = client
            .login(&self.config.username, &self.config.password)
            .await
            .map_err(|e| format!("Failed to login to IMAP: {:?}", e.0))?;

        Ok(session)
    }

    fn parse_message(fetch: &async_imap::types::Fetch) -> Result<EmailMessage, String> {
        let envelope = fetch
            .envelope()
            .ok_or_else(|| "No envelope in message".to_string())?;

        let subject = envelope
            .subject
            .as_ref()
            .and_then(|s| std::str::from_utf8(s).ok())
            .unwrap_or("")
            .to_string();

        let from = envelope
            .from
            .as_ref()
            .and_then(|addrs| addrs.first())
            .and_then(|addr| {
                let mailbox = addr.mailbox.as_ref().and_then(|m| std::str::from_utf8(m).ok())?;
                let host = addr.host.as_ref().and_then(|h| std::str::from_utf8(h).ok())?;
                Some(format!("{}@{}", mailbox, host))
            })
            .unwrap_or_default();

        let to = envelope
            .to
            .as_ref()
            .map(|addrs| {
                addrs
                    .iter()
                    .filter_map(|addr| {
                        let mailbox = addr.mailbox.as_ref().and_then(|m| std::str::from_utf8(m).ok())?;
                        let host = addr.host.as_ref().and_then(|h| std::str::from_utf8(h).ok())?;
                        Some(format!("{}@{}", mailbox, host))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let date = envelope
            .date
            .as_ref()
            .and_then(|d| std::str::from_utf8(d).ok())
            .map(|s| s.to_string());

        let body_text = fetch.text().and_then(|b| std::str::from_utf8(b).ok()).map(|s| s.to_string());

        let is_seen = fetch.flags().any(|f| matches!(f, async_imap::types::Flag::Seen));

        Ok(EmailMessage {
            uid: fetch.uid.unwrap_or(0),
            subject,
            from,
            to,
            date,
            body_text,
            body_html: None,
            is_seen,
        })
    }
}

#[cfg(feature = "email-imap")]
#[async_trait]
impl ImapProvider for AsyncImapProvider {
    async fn fetch_messages(&self, limit: Option<u32>) -> Result<Vec<EmailMessage>, String> {
        let mut session = self.connect().await?;

        session
            .select("INBOX")
            .await
            .map_err(|e| format!("Failed to select INBOX: {}", e))?;

        let sequence = if let Some(limit) = limit {
            format!("1:{}", limit)
        } else {
            "1:*".to_string()
        };

        let messages_stream = session
            .fetch(sequence, "(ENVELOPE FLAGS BODY[TEXT])")
            .await
            .map_err(|e| format!("Failed to fetch messages: {}", e))?;

        let mut result = Vec::new();
        let fetches: Vec<_> = messages_stream
            .collect::<Vec<_>>()
            .await;

        for fetch_result in fetches {
            match fetch_result {
                Ok(fetch) => {
                    if let Ok(msg) = Self::parse_message(&fetch) {
                        result.push(msg);
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching message: {}", e);
                }
            }
        }

        session
            .logout()
            .await
            .map_err(|e| format!("Failed to logout: {}", e))?;

        Ok(result)
    }

    async fn search_messages(&self, query: &str) -> Result<Vec<EmailMessage>, String> {
        let mut session = self.connect().await?;

        session
            .select("INBOX")
            .await
            .map_err(|e| format!("Failed to select INBOX: {}", e))?;

        let uids = session
            .search(query)
            .await
            .map_err(|e| format!("Failed to search: {}", e))?;

        if uids.is_empty() {
            session.logout().await.ok();
            return Ok(Vec::new());
        }

        let uid_sequence = uids
            .iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let messages_stream = session
            .fetch(uid_sequence, "(ENVELOPE FLAGS BODY[TEXT])")
            .await
            .map_err(|e| format!("Failed to fetch messages: {}", e))?;

        let mut result = Vec::new();
        let fetches: Vec<_> = messages_stream
            .collect::<Vec<_>>()
            .await;

        for fetch_result in fetches {
            match fetch_result {
                Ok(fetch) => {
                    if let Ok(msg) = Self::parse_message(&fetch) {
                        result.push(msg);
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching message: {}", e);
                }
            }
        }

        session
            .logout()
            .await
            .map_err(|e| format!("Failed to logout: {}", e))?;

        Ok(result)
    }

    async fn mark_seen(&self, uid: u32) -> Result<(), String> {
        let mut session = self.connect().await?;

        session
            .select("INBOX")
            .await
            .map_err(|e| format!("Failed to select INBOX: {}", e))?;

        session
            .store(format!("{}", uid), "+FLAGS (\\Seen)")
            .await
            .map_err(|e| format!("Failed to mark as seen: {}", e))?;

        session
            .logout()
            .await
            .map_err(|e| format!("Failed to logout: {}", e))?;

        Ok(())
    }

    async fn mark_unseen(&self, uid: u32) -> Result<(), String> {
        let mut session = self.connect().await?;

        session
            .select("INBOX")
            .await
            .map_err(|e| format!("Failed to select INBOX: {}", e))?;

        session
            .store(format!("{}", uid), "-FLAGS (\\Seen)")
            .await
            .map_err(|e| format!("Failed to mark as unseen: {}", e))?;

        session
            .logout()
            .await
            .map_err(|e| format!("Failed to logout: {}", e))?;

        Ok(())
    }
}

#[cfg(feature = "email-imap")]
pub fn create_imap_provider(config: ImapAccountConfig) -> Box<dyn ImapProvider> {
    Box::new(AsyncImapProvider::new(config))
}

#[cfg(not(feature = "email-imap"))]
pub fn create_imap_provider(_config: ImapAccountConfig) -> Box<dyn ImapProvider> {
    panic!("IMAP support not enabled. Enable 'email-imap' feature.");
}
