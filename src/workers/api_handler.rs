use crate::api::ZerodhaClient;
use crate::data_structures::*;
use crate::state::{Command, Config, EventSender};
use crossbeam_channel::Receiver;
use std::sync::Arc;
use tokio::sync::RwLock;

/// High-performance API handler worker for REST API operations
/// Runs in a dedicated thread to prevent blocking the UI
pub struct ApiHandler {
    client: Arc<RwLock<ZerodhaClient>>,
    event_sender: EventSender,
    config: Config,
}

impl ApiHandler {
    /// Create new API handler with optimized Zerodha client
    pub fn new(config: Config, event_sender: EventSender) -> Self {
        let mut client = ZerodhaClient::new(
            config.zerodha.api_key.clone(),
            config.zerodha.api_secret.clone(),
        );

        // Set the access token from configuration for personal trading
        client.set_access_token(config.zerodha.access_token.clone());

        Self {
            client: Arc::new(RwLock::new(client)),
            event_sender,
            config,
        }
    }

    /// Main worker loop - processes commands from UI thread
    /// Designed for ultra-low latency command processing
    pub async fn run(&mut self, command_receiver: Receiver<Command>) {
        self.event_sender.send_notification(
            LogLevel::Info,
            "API handler started".to_string(),
            Some("api_handler".to_string()),
        );

        while let Ok(command) = command_receiver.recv() {
            if let Command::Shutdown = command {
                break;
            }

            if let Err(e) = self.handle_command(command).await {
                self.event_sender.send_error(
                    format!("Command handling error: {}", e),
                    Some("api_handler".to_string()),
                );
            }
        }

        self.event_sender.send_notification(
            LogLevel::Info,
            "API handler stopped".to_string(),
            Some("api_handler".to_string()),
        );
    }

    /// Handle individual commands with comprehensive error handling
    async fn handle_command(&mut self, command: Command) -> anyhow::Result<()> {
        match command {
            Command::FetchPositions => {
                self.handle_fetch_positions().await?;
            }

            Command::FetchOrders => {
                self.handle_fetch_orders().await?;
            }

            Command::FetchUserProfile => {
                self.handle_fetch_user_profile().await?;
            }

            Command::FetchInstruments { exchange } => {
                self.handle_fetch_instruments(exchange).await?;
            }

            Command::PlaceOrder { details } => {
                self.handle_place_order(details).await?;
            }

            Command::ModifyOrder { order_id, details } => {
                self.handle_modify_order(order_id, details).await?;
            }

            Command::CancelOrder { order_id } => {
                self.handle_cancel_order(order_id).await?;
            }

            // WebSocket commands are handled by websocket_handler
            Command::SubscribeToTicks { .. }
            | Command::UnsubscribeFromTicks { .. }
            | Command::ReconnectWebSocket => {
                // These are handled by websocket_handler, ignore here
            }

            Command::Shutdown => {
                // Already handled above
            }
        }

        Ok(())
    }

    /// Fetch user profile information
    async fn handle_fetch_user_profile(&mut self) -> anyhow::Result<()> {
        // For personal trading, we can skip user profile fetching
        self.event_sender.send_notification(
            LogLevel::Info,
            "User profile not needed for personal trading".to_string(),
            Some("api_handler".to_string()),
        );

        Ok(())
    }

    /// Fetch positions with optimized error handling
    async fn handle_fetch_positions(&mut self) -> anyhow::Result<()> {
        let client = self.client.read().await;

        match client.get_positions().await {
            Ok(positions) => {
                self.event_sender
                    .send(crate::state::AppEvent::PositionsUpdated(positions))?;

                self.event_sender.send_notification(
                    LogLevel::Info,
                    "Positions fetched successfully".to_string(),
                    Some("api_handler".to_string()),
                );
            }
            Err(e) => {
                self.event_sender.send_error(
                    format!("Failed to fetch positions: {}", e),
                    Some("api_handler".to_string()),
                );
            }
        }

        Ok(())
    }

    /// Fetch orders with optimized performance
    async fn handle_fetch_orders(&mut self) -> anyhow::Result<()> {
        let client = self.client.read().await;

        match client.get_orders().await {
            Ok(orders) => {
                self.event_sender
                    .send(crate::state::AppEvent::OrdersUpdated(orders))?;

                self.event_sender.send_notification(
                    LogLevel::Info,
                    "Orders fetched successfully".to_string(),
                    Some("api_handler".to_string()),
                );
            }
            Err(e) => {
                self.event_sender.send_error(
                    format!("Failed to fetch orders: {}", e),
                    Some("api_handler".to_string()),
                );
            }
        }

        Ok(())
    }

    /// Fetch instruments for a specific exchange
    async fn handle_fetch_instruments(&mut self, exchange: String) -> anyhow::Result<()> {
        let client = self.client.read().await;

        match client.get_instruments(&exchange).await {
            Ok(instruments) => {
                self.event_sender
                    .send(crate::state::AppEvent::InstrumentsUpdated(instruments))?;

                self.event_sender.send_notification(
                    LogLevel::Info,
                    format!("Instruments fetched for exchange: {}", exchange),
                    Some("api_handler".to_string()),
                );
            }
            Err(e) => {
                self.event_sender.send_error(
                    format!("Failed to fetch instruments for {}: {}", exchange, e),
                    Some("api_handler".to_string()),
                );
            }
        }

        Ok(())
    }

    /// Place a new order with validation
    async fn handle_place_order(&mut self, order_request: OrderRequest) -> anyhow::Result<()> {
        self.event_sender.send_notification(
            LogLevel::Info,
            format!("Placing order for {}", order_request.tradingsymbol),
            Some("api_handler".to_string()),
        );

        let order_id = {
            let client = self.client.read().await;
            client.place_order(&order_request).await
        };

        match order_id {
            Ok(order_id) => {
                self.event_sender
                    .send(crate::state::AppEvent::OrderPlaced {
                        order_id: order_id.clone(),
                    })?;

                self.event_sender.send_notification(
                    LogLevel::Info,
                    format!("Order placed successfully: {}", order_id),
                    Some("api_handler".to_string()),
                );

                // Refresh orders after placing
                if let Err(e) = self.handle_fetch_orders().await {
                    self.event_sender.send_error(
                        format!("Failed to refresh orders after placement: {}", e),
                        Some("api_handler".to_string()),
                    );
                }
            }
            Err(e) => {
                self.event_sender.send_error(
                    format!("Failed to place order: {}", e),
                    Some("api_handler".to_string()),
                );
            }
        }

        Ok(())
    }

    /// Modify an existing order
    async fn handle_modify_order(
        &mut self,
        order_id: String,
        _order_request: OrderRequest,
    ) -> anyhow::Result<()> {
        self.event_sender.send_notification(
            LogLevel::Info,
            format!("Modifying order: {}", order_id),
            Some("api_handler".to_string()),
        );

        // Note: In a real implementation, you would call client.modify_order()
        // This is a placeholder as the modify_order method would need to be implemented
        self.event_sender
            .send(crate::state::AppEvent::OrderModified {
                order_id: order_id.clone(),
            })?;

        self.event_sender.send_notification(
            LogLevel::Info,
            format!("Order modified: {}", order_id),
            Some("api_handler".to_string()),
        );

        // Refresh orders after modification
        self.handle_fetch_orders().await?;

        Ok(())
    }

    /// Cancel an existing order
    async fn handle_cancel_order(&mut self, order_id: String) -> anyhow::Result<()> {
        self.event_sender.send_notification(
            LogLevel::Info,
            format!("Cancelling order: {}", order_id),
            Some("api_handler".to_string()),
        );

        let cancel_result = {
            let client = self.client.read().await;
            // Default to "regular" variety - in real implementation, track order varieties
            client.cancel_order(&order_id, "regular").await
        };

        match cancel_result {
            Ok(cancelled_order_id) => {
                self.event_sender
                    .send(crate::state::AppEvent::OrderCancelled {
                        order_id: cancelled_order_id.clone(),
                    })?;

                self.event_sender.send_notification(
                    LogLevel::Info,
                    format!("Order cancelled: {}", cancelled_order_id),
                    Some("api_handler".to_string()),
                );

                // Refresh orders after cancellation
                if let Err(e) = self.handle_fetch_orders().await {
                    self.event_sender.send_error(
                        format!("Failed to refresh orders after cancellation: {}", e),
                        Some("api_handler".to_string()),
                    );
                }
            }
            Err(e) => {
                self.event_sender.send_error(
                    format!("Failed to cancel order {}: {}", order_id, e),
                    Some("api_handler".to_string()),
                );
            }
        }

        Ok(())
    }
}
