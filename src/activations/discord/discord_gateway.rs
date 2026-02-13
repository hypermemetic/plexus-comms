use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream, MaybeTlsStream};
use tokio::net::TcpStream;

const GATEWAY_URL: &str = "wss://gateway.discord.gg/?v=10&encoding=json";
const INTENTS: u32 = (1 << 9) | (1 << 15); // GUILD_MESSAGES | MESSAGE_CONTENT = 33280

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub struct DiscordGateway {
    bot_token: String,
    websocket: Arc<Mutex<Option<WsStream>>>,
    heartbeat_interval: Arc<Mutex<Option<u64>>>,
    sequence: Arc<Mutex<Option<u64>>>,
    session_id: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Deserialize)]
struct GatewayPayload {
    op: u8,
    d: Option<serde_json::Value>,
    s: Option<u64>,
    t: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HelloData {
    heartbeat_interval: u64,
}

#[derive(Debug, Deserialize)]
struct ReadyData {
    session_id: String,
    user: serde_json::Value,
}

// Gateway event data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayMessage {
    pub id: String,
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub author: GatewayUser,
    pub content: String,
    pub timestamp: String,
    pub edited_timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub bot: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayMember {
    pub user: Option<GatewayUser>,
    pub guild_id: String,
    pub joined_at: String,
}

#[derive(Debug, Clone)]
pub enum GatewayEvent {
    MessageCreate(GatewayMessage),
    MessageUpdate(GatewayMessage),
    MessageDelete {
        id: String,
        channel_id: String,
        guild_id: Option<String>,
    },
    GuildMemberAdd(GatewayMember),
    Ready {
        session_id: String,
        user: serde_json::Value,
    },
    Error(String),
    Disconnected,
}

impl DiscordGateway {
    pub fn new(bot_token: String) -> Self {
        Self {
            bot_token,
            websocket: Arc::new(Mutex::new(None)),
            heartbeat_interval: Arc::new(Mutex::new(None)),
            sequence: Arc::new(Mutex::new(None)),
            session_id: Arc::new(Mutex::new(None)),
        }
    }

    /// Connect to Discord Gateway and start receiving events
    pub async fn connect(&self) -> Result<(), String> {
        tracing::info!("Connecting to Discord Gateway...");

        let (ws_stream, _) = connect_async(GATEWAY_URL)
            .await
            .map_err(|e| format!("Failed to connect to Gateway: {}", e))?;

        *self.websocket.lock().await = Some(ws_stream);

        tracing::info!("Connected to Discord Gateway");
        Ok(())
    }

    /// Send IDENTIFY payload to authenticate
    async fn send_identify(&self) -> Result<(), String> {
        let identify = json!({
            "op": 2,
            "d": {
                "token": self.bot_token,
                "intents": INTENTS,
                "properties": {
                    "$os": "linux",
                    "$browser": "plexus-comms",
                    "$device": "plexus-comms"
                }
            }
        });

        self.send_payload(identify).await
    }

    /// Send RESUME payload to reconnect
    async fn send_resume(&self) -> Result<(), String> {
        let session_id = self.session_id.lock().await.clone();
        let sequence = self.sequence.lock().await.clone();

        match (session_id, sequence) {
            (Some(sid), Some(seq)) => {
                let resume = json!({
                    "op": 6,
                    "d": {
                        "token": self.bot_token,
                        "session_id": sid,
                        "seq": seq
                    }
                });
                self.send_payload(resume).await
            }
            _ => {
                tracing::warn!("Cannot resume: missing session_id or sequence");
                self.send_identify().await
            }
        }
    }

    /// Send HEARTBEAT payload
    async fn send_heartbeat(&self) -> Result<(), String> {
        let sequence = self.sequence.lock().await.clone();
        let heartbeat = json!({
            "op": 1,
            "d": sequence
        });

        self.send_payload(heartbeat).await
    }

    /// Send a JSON payload to the Gateway
    async fn send_payload(&self, payload: serde_json::Value) -> Result<(), String> {
        let mut ws_lock = self.websocket.lock().await;
        if let Some(ws) = ws_lock.as_mut() {
            let message = Message::Text(payload.to_string());
            ws.send(message)
                .await
                .map_err(|e| format!("Failed to send payload: {}", e))?;
            Ok(())
        } else {
            Err("WebSocket not connected".to_string())
        }
    }

    /// Main event loop - handles Gateway messages and events
    pub async fn run(
        &self,
        mut event_tx: tokio::sync::mpsc::UnboundedSender<GatewayEvent>,
    ) -> Result<(), String> {
        // Connect to Gateway
        self.connect().await?;

        // Spawn heartbeat task
        let heartbeat_handle = self.spawn_heartbeat_task();

        // Main message loop
        loop {
            let msg = {
                let mut ws_lock = self.websocket.lock().await;
                if let Some(ws) = ws_lock.as_mut() {
                    match ws.next().await {
                        Some(Ok(msg)) => msg,
                        Some(Err(e)) => {
                            tracing::error!("WebSocket error: {}", e);
                            let _ = event_tx.send(GatewayEvent::Error(format!("WebSocket error: {}", e)));
                            break;
                        }
                        None => {
                            tracing::info!("WebSocket connection closed");
                            let _ = event_tx.send(GatewayEvent::Disconnected);
                            break;
                        }
                    }
                } else {
                    break;
                }
            };

            match msg {
                Message::Text(text) => {
                    if let Err(e) = self.handle_message(&text, &mut event_tx).await {
                        tracing::error!("Failed to handle message: {}", e);
                        let _ = event_tx.send(GatewayEvent::Error(e));
                    }
                }
                Message::Close(_) => {
                    tracing::info!("Received close frame from Gateway");
                    let _ = event_tx.send(GatewayEvent::Disconnected);
                    break;
                }
                _ => {}
            }
        }

        // Cleanup
        heartbeat_handle.abort();
        Ok(())
    }

    /// Handle incoming Gateway message
    async fn handle_message(
        &self,
        text: &str,
        event_tx: &mut tokio::sync::mpsc::UnboundedSender<GatewayEvent>,
    ) -> Result<(), String> {
        let payload: GatewayPayload = serde_json::from_str(text)
            .map_err(|e| format!("Failed to parse Gateway payload: {}", e))?;

        // Update sequence number
        if let Some(s) = payload.s {
            *self.sequence.lock().await = Some(s);
        }

        match payload.op {
            // DISPATCH - Event
            0 => {
                if let (Some(event_type), Some(data)) = (payload.t, payload.d) {
                    self.handle_dispatch_event(&event_type, data, event_tx).await?;
                }
            }
            // HEARTBEAT - Server requesting heartbeat
            1 => {
                self.send_heartbeat().await?;
            }
            // RECONNECT - Server requesting reconnect
            7 => {
                tracing::info!("Gateway requested reconnect");
                // Client should reconnect and resume
            }
            // INVALID SESSION - Session is invalid
            9 => {
                tracing::warn!("Invalid session, re-identifying...");
                tokio::time::sleep(Duration::from_secs(2)).await;
                self.send_identify().await?;
            }
            // HELLO - Initial handshake
            10 => {
                if let Some(data) = payload.d {
                    let hello: HelloData = serde_json::from_value(data)
                        .map_err(|e| format!("Failed to parse HELLO data: {}", e))?;

                    *self.heartbeat_interval.lock().await = Some(hello.heartbeat_interval);
                    tracing::info!("Received HELLO, heartbeat_interval: {}ms", hello.heartbeat_interval);

                    // Send IDENTIFY to authenticate
                    self.send_identify().await?;
                }
            }
            // HEARTBEAT ACK
            11 => {
                tracing::debug!("Received heartbeat ACK");
            }
            _ => {
                tracing::debug!("Unhandled opcode: {}", payload.op);
            }
        }

        Ok(())
    }

    /// Handle DISPATCH events (opcode 0)
    async fn handle_dispatch_event(
        &self,
        event_type: &str,
        data: serde_json::Value,
        event_tx: &mut tokio::sync::mpsc::UnboundedSender<GatewayEvent>,
    ) -> Result<(), String> {
        match event_type {
            "READY" => {
                let ready: ReadyData = serde_json::from_value(data)
                    .map_err(|e| format!("Failed to parse READY data: {}", e))?;

                *self.session_id.lock().await = Some(ready.session_id.clone());
                tracing::info!("Gateway READY, session_id: {}", ready.session_id);

                let _ = event_tx.send(GatewayEvent::Ready {
                    session_id: ready.session_id,
                    user: ready.user,
                });
            }
            "MESSAGE_CREATE" => {
                let message: GatewayMessage = serde_json::from_value(data)
                    .map_err(|e| format!("Failed to parse MESSAGE_CREATE: {}", e))?;

                let _ = event_tx.send(GatewayEvent::MessageCreate(message));
            }
            "MESSAGE_UPDATE" => {
                let message: GatewayMessage = serde_json::from_value(data)
                    .map_err(|e| format!("Failed to parse MESSAGE_UPDATE: {}", e))?;

                let _ = event_tx.send(GatewayEvent::MessageUpdate(message));
            }
            "MESSAGE_DELETE" => {
                let id = data["id"].as_str().unwrap_or("").to_string();
                let channel_id = data["channel_id"].as_str().unwrap_or("").to_string();
                let guild_id = data["guild_id"].as_str().map(|s| s.to_string());

                let _ = event_tx.send(GatewayEvent::MessageDelete {
                    id,
                    channel_id,
                    guild_id,
                });
            }
            "GUILD_MEMBER_ADD" => {
                let member: GatewayMember = serde_json::from_value(data)
                    .map_err(|e| format!("Failed to parse GUILD_MEMBER_ADD: {}", e))?;

                let _ = event_tx.send(GatewayEvent::GuildMemberAdd(member));
            }
            _ => {
                tracing::debug!("Unhandled event type: {}", event_type);
            }
        }

        Ok(())
    }

    /// Spawn a task to send periodic heartbeats
    fn spawn_heartbeat_task(&self) -> tokio::task::JoinHandle<()> {
        let heartbeat_interval = self.heartbeat_interval.clone();
        let gateway = Self {
            bot_token: self.bot_token.clone(),
            websocket: self.websocket.clone(),
            heartbeat_interval: self.heartbeat_interval.clone(),
            sequence: self.sequence.clone(),
            session_id: self.session_id.clone(),
        };

        tokio::spawn(async move {
            // Wait for heartbeat interval to be set
            loop {
                let interval_ms = heartbeat_interval.lock().await.clone();
                if let Some(ms) = interval_ms {
                    tracing::info!("Starting heartbeat task with interval: {}ms", ms);
                    let mut ticker = interval(Duration::from_millis(ms));

                    loop {
                        ticker.tick().await;
                        if let Err(e) = gateway.send_heartbeat().await {
                            tracing::error!("Failed to send heartbeat: {}", e);
                            break;
                        }
                        tracing::debug!("Sent heartbeat");
                    }
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
    }

    /// Close the Gateway connection
    pub async fn close(&self) -> Result<(), String> {
        let mut ws_lock = self.websocket.lock().await;
        if let Some(ws) = ws_lock.as_mut() {
            ws.close(None)
                .await
                .map_err(|e| format!("Failed to close WebSocket: {}", e))?;
        }
        *ws_lock = None;
        Ok(())
    }
}
