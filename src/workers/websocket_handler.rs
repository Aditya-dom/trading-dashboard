use crate::data_structures::*;
use crate::state::{Command, Config, EventSender};
use chrono::Utc;
use crossbeam_channel::Receiver;
use futures_util::{SinkExt, StreamExt};
use reqwest;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Ultra-high-performance WebSocket handler for real-time market data
/// Optimized for minimal latency tick processing using zero-copy deserialization
pub struct WebSocketHandler {
    event_sender: EventSender,
    config: Config,
    access_token: Arc<RwLock<Option<String>>>,
    subscribed_tokens: Arc<RwLock<Vec<u32>>>,
    reconnect_attempts: u32,
    is_connected: Arc<RwLock<bool>>,
}

impl WebSocketHandler {
    /// Create new WebSocket handler with optimized configuration
    pub fn new(config: Config, event_sender: EventSender) -> Self {
        // Only use access token if it's not the placeholder
        let access_token = if config.zerodha.access_token != "your_access_token_here" 
            && !config.zerodha.access_token.is_empty() {
            Some(config.zerodha.access_token.clone())
        } else {
            None
        };
        
        let access_token = Arc::new(RwLock::new(access_token));

        Self {
            event_sender,
            config,
            access_token,
            subscribed_tokens: Arc::new(RwLock::new(Vec::new())),
            reconnect_attempts: 0,
            is_connected: Arc::new(RwLock::new(false)),
        }
    }

    /// Set access token for WebSocket authentication
    pub async fn set_access_token(&self, token: String) {
        let mut access_token = self.access_token.write().await;
        *access_token = Some(token);
    }

    /// Main worker loop - handles WebSocket connections and tick processing
    /// Designed for ultra-low latency real-time data processing
    pub async fn run(&mut self, command_receiver: Receiver<Command>) {
        self.event_sender.send_notification(
            LogLevel::Info,
            "WebSocket handler started".to_string(),
            Some("websocket_handler".to_string()),
        );

        // Start command processing task
        let command_receiver_clone = command_receiver.clone();
        let self_clone = self.clone_for_task();
        tokio::spawn(async move {
            self_clone.command_processor(command_receiver_clone).await;
        });

        // Main WebSocket connection loop with auto-reconnect
        loop {
            let token = {
                let access_token_guard = self.access_token.read().await;
                access_token_guard.clone()
            };

            if let Some(token) = token {
                if let Err(e) = self.connect_and_process(&token).await {
                    self.event_sender.send_error(
                        format!("WebSocket connection error: {}", e),
                        Some("websocket_handler".to_string()),
                    );

                    self.handle_reconnection().await;
                } else {
                    // Connection closed gracefully
                    break;
                }
            } else {
                // Wait for access token
                sleep(Duration::from_millis(100)).await;
            }
        }

        self.event_sender.send_notification(
            LogLevel::Info,
            "WebSocket handler stopped".to_string(),
            Some("websocket_handler".to_string()),
        );
    }

    /// Clone self for task spawning (simplified version)
    fn clone_for_task(&self) -> Self {
        Self {
            event_sender: self.event_sender.clone(),
            config: self.config.clone(),
            access_token: Arc::clone(&self.access_token),
            subscribed_tokens: Arc::clone(&self.subscribed_tokens),
            reconnect_attempts: 0,
            is_connected: Arc::clone(&self.is_connected),
        }
    }

    /// Process commands from UI thread
    async fn command_processor(&self, command_receiver: Receiver<Command>) {
        while let Ok(command) = command_receiver.recv() {
            match command {
                Command::SubscribeToTicks { instrument_tokens } => {
                    self.handle_subscribe(instrument_tokens).await;
                }

                Command::UnsubscribeFromTicks { instrument_tokens } => {
                    self.handle_unsubscribe(instrument_tokens).await;
                }

                Command::ReconnectWebSocket => {
                    self.event_sender.send_notification(
                        LogLevel::Info,
                        "Manual reconnection requested".to_string(),
                        Some("websocket_handler".to_string()),
                    );
                    // The main loop will handle reconnection
                }

                Command::Shutdown => {
                    break;
                }

                // Other commands are handled by API handler
                _ => {}
            }
        }
    }

    /// Establish WebSocket connection and process incoming messages
    async fn connect_and_process(&mut self, access_token: &str) -> anyhow::Result<()> {
        // DEBUGGING: Confirm we're using the updated code
        println!("🚀 USING UPDATED WEBSOCKET HANDLER - Fixed URL format");
        
        // FIXED: Remove extra slash - Zerodha requires exact format
        let ws_url = format!(
            "wss://ws.kite.trade?api_key={}&access_token={}",
            self.config.zerodha.api_key, access_token
        );
        
        println!("🔗 WebSocket URL: {}", ws_url);
        println!("🔑 Using API Key: {}", self.config.zerodha.api_key);
        println!("🎫 Access Token: {}...", &access_token[..8]);

        self.event_sender.send_notification(
            LogLevel::Info,
            "Connecting to Zerodha WebSocket...".to_string(),
            Some("websocket_handler".to_string()),
        );

        // First, validate the access token with a REST API call
        println!("🧪 Testing access token with REST API...");
        let test_client = reqwest::Client::new();
        let test_response = test_client
            .get("https://api.kite.trade/user/profile")
            .header("X-Kite-Version", "3")
            .header("Authorization", format!("token {}:{}", self.config.zerodha.api_key, access_token))
            .send()
            .await;
            
        match test_response {
            Ok(resp) if resp.status() == 200 => {
                println!("✅ Access token is valid - proceeding with WebSocket connection");
            }
            Ok(resp) => {
                let status = resp.status();
                let error_text = resp.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!("❌ REST API test failed: {} - {}", status, error_text));
            }
            Err(e) => {
                return Err(anyhow::anyhow!("❌ REST API test failed: {}", e));
            }
        }

        // Connect with extended timeout
        let connect_result = timeout(Duration::from_secs(30), connect_async(&ws_url)).await;

        let (ws_stream, _) = match connect_result {
            Ok(Ok(connection)) => connection,
            Ok(Err(e)) => {
                return Err(anyhow::anyhow!("WebSocket connection failed: {}", e));
            }
            Err(_) => {
                return Err(anyhow::anyhow!("WebSocket connection timeout"));
            }
        };

        // Connection successful
        {
            let mut is_connected = self.is_connected.write().await;
            *is_connected = true;
        }

        self.reconnect_attempts = 0;
        self.event_sender
            .send(crate::state::AppEvent::WebSocketConnected)?;

        self.event_sender.send_notification(
            LogLevel::Info,
            "WebSocket connected successfully".to_string(),
            Some("websocket_handler".to_string()),
        );

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Subscribe to existing tokens
        let tokens = self.subscribed_tokens.read().await.clone();
        if !tokens.is_empty() {
            self.send_subscription(&mut ws_sender, &tokens).await?;
        }

        // Process incoming messages with high-frequency optimization
        while let Some(msg_result) = ws_receiver.next().await {
            match msg_result {
                Ok(Message::Binary(data)) => {
                    // High-frequency tick processing using zero-copy deserialization
                    if let Err(e) = self.process_tick_data(&data).await {
                        self.event_sender.send_error(
                            format!("Tick processing error: {}", e),
                            Some("websocket_handler".to_string()),
                        );
                    }
                }

                Ok(Message::Text(text)) => {
                    // Handle text messages (usually status updates)
                    self.process_text_message(&text).await;
                }

                Ok(Message::Close(_)) => {
                    self.event_sender.send_notification(
                        LogLevel::Info,
                        "WebSocket connection closed by server".to_string(),
                        Some("websocket_handler".to_string()),
                    );
                    break;
                }

                Ok(Message::Ping(_)) => {
                    // Respond to ping with pong
                    if let Err(e) = ws_sender.send(Message::Pong(vec![].into())).await {
                        self.event_sender.send_error(
                            format!("Failed to send pong: {}", e),
                            Some("websocket_handler".to_string()),
                        );
                    }
                }

                Ok(_) => {
                    // Ignore other message types
                }

                Err(e) => {
                    return Err(anyhow::anyhow!("WebSocket error: {}", e));
                }
            }
        }

        // Connection lost
        {
            let mut is_connected = self.is_connected.write().await;
            *is_connected = false;
        }

        self.event_sender
            .send(crate::state::AppEvent::WebSocketDisconnected)?;

        Ok(())
    }

    /// Handle subscription to instrument tokens
    async fn handle_subscribe(&self, instrument_tokens: Vec<u32>) {
        let mut tokens = self.subscribed_tokens.write().await;

        // Add new tokens (avoid duplicates)
        for token in &instrument_tokens {
            if !tokens.contains(token) {
                tokens.push(*token);
            }
        }

        self.event_sender.send_notification(
            LogLevel::Info,
            format!("Subscribed to {} tokens", instrument_tokens.len()),
            Some("websocket_handler".to_string()),
        );

        // If connected, send subscription immediately
        // Note: In a real implementation, you would need access to the WebSocket sender here
        // This would require a more complex architecture with channels
    }

    /// Handle unsubscription from instrument tokens
    async fn handle_unsubscribe(&self, instrument_tokens: Vec<u32>) {
        let mut tokens = self.subscribed_tokens.write().await;

        // Remove tokens
        tokens.retain(|token| !instrument_tokens.contains(token));

        self.event_sender.send_notification(
            LogLevel::Info,
            format!("Unsubscribed from {} tokens", instrument_tokens.len()),
            Some("websocket_handler".to_string()),
        );
    }

    /// Send subscription message to WebSocket
    async fn send_subscription(
        &self,
        ws_sender: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
        tokens: &[u32],
    ) -> anyhow::Result<()> {
        // Zerodha WebSocket subscription format
        let subscription_msg = serde_json::json!({
            "a": "subscribe",
            "v": tokens
        });

        let msg_text = serde_json::to_string(&subscription_msg)?;

        ws_sender.send(Message::Text(msg_text.into())).await?;

        self.event_sender.send_notification(
            LogLevel::Info,
            format!("Sent subscription for {} tokens", tokens.len()),
            Some("websocket_handler".to_string()),
        );

        Ok(())
    }

    /// Process binary tick data with zero-copy deserialization for ultra-low latency
    async fn process_tick_data(&self, data: &[u8]) -> anyhow::Result<()> {
        // Zerodha tick format parsing
        // Note: This is a simplified implementation. Real Zerodha ticks have a specific binary format
        // that would require proper parsing according to their documentation

        if data.len() < 8 {
            return Ok(()); // Invalid tick data
        }

        // Parse instrument token (first 4 bytes, big-endian)
        let instrument_token = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);

        // Parse last price (next 4 bytes as f32, then convert to f64)
        let last_price_raw = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let last_price = f32::from_bits(last_price_raw) as f64;

        // For volume, we'd need more bytes - simplified here
        let volume = if data.len() >= 16 {
            u64::from_be_bytes([
                data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15],
            ])
        } else {
            0
        };

        // Send tick update event
        self.event_sender.send(crate::state::AppEvent::TickUpdate {
            instrument_token,
            last_price,
            volume,
            timestamp: Utc::now(),
        })?;

        Ok(())
    }

    /// Process text messages from WebSocket
    async fn process_text_message(&self, text: &str) {
        // Parse status messages, connection confirmations, etc.
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(text) {
            if let Some(status) = json_value.get("status") {
                self.event_sender.send_notification(
                    LogLevel::Info,
                    format!("WebSocket status: {}", status),
                    Some("websocket_handler".to_string()),
                );
            }
        } else {
            self.event_sender.send_notification(
                LogLevel::Debug,
                format!("Received text message: {}", text),
                Some("websocket_handler".to_string()),
            );
        }
    }

    /// Handle reconnection with exponential backoff
    async fn handle_reconnection(&mut self) {
        self.reconnect_attempts += 1;

        if self.reconnect_attempts > self.config.app.max_reconnect_attempts {
            self.event_sender.send_error(
                "Max reconnection attempts reached".to_string(),
                Some("websocket_handler".to_string()),
            );
            return;
        }

        self.event_sender
            .send(crate::state::AppEvent::WebSocketReconnecting {
                attempt: self.reconnect_attempts,
            })
            .unwrap_or_else(|_| {});

        // Exponential backoff with jitter
        let base_delay = self.config.app.websocket_reconnect_delay_ms;
        let delay = base_delay * (2_u64.pow(self.reconnect_attempts.min(5)));
        let jitter = fastrand::u64(0..delay / 4); // Add up to 25% jitter
        let total_delay = delay + jitter;

        self.event_sender.send_notification(
            LogLevel::Info,
            format!(
                "Reconnecting in {}ms (attempt {})",
                total_delay, self.reconnect_attempts
            ),
            Some("websocket_handler".to_string()),
        );

        sleep(Duration::from_millis(total_delay)).await;
    }
}

impl Clone for WebSocketHandler {
    fn clone(&self) -> Self {
        Self {
            event_sender: self.event_sender.clone(),
            config: self.config.clone(),
            access_token: Arc::clone(&self.access_token),
            subscribed_tokens: Arc::clone(&self.subscribed_tokens),
            reconnect_attempts: self.reconnect_attempts,
            is_connected: Arc::clone(&self.is_connected),
        }
    }
}
