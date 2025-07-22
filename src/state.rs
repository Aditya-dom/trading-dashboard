use crate::data_structures::*;
use chrono::{DateTime, Utc};
use crossbeam_channel::{Receiver, Sender};
use dashmap::DashMap;
use figment::providers::Format;
use figment::{providers::Toml, Figment};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Configuration structure mirroring config.toml for type-safe access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub zerodha: ZerodhaConfig,
    pub app: AppConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZerodhaConfig {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub log_level: String,
    pub websocket_reconnect_delay_ms: u64,
    pub max_reconnect_attempts: u32,
    pub tick_buffer_size: usize,
}

impl Config {
    /// Load configuration from config.toml with comprehensive error handling
    pub fn load() -> Result<Self, figment::Error> {
        Figment::new().merge(Toml::file("config.toml")).extract()
    }
}

/// Authentication states for the trading application
#[derive(Debug, Clone)]
pub enum AuthState {
    LoggedIn {
        access_token: String,
        user_name: String,
        user_profile: Option<UserProfile>,
    },
}

/// Commands sent from UI thread to worker threads
/// Designed for zero-allocation message passing using crossbeam-channel
#[derive(Debug, Clone)]
pub enum Command {
    // Data fetching commands
    FetchPositions,
    FetchOrders,
    FetchUserProfile,
    FetchInstruments {
        exchange: String,
    },

    // WebSocket commands
    SubscribeToTicks {
        instrument_tokens: Vec<u32>,
    },
    UnsubscribeFromTicks {
        instrument_tokens: Vec<u32>,
    },

    // Trading commands
    PlaceOrder {
        details: OrderRequest,
    },
    ModifyOrder {
        order_id: String,
        details: OrderRequest,
    },
    CancelOrder {
        order_id: String,
    },

    // Connection management
    ReconnectWebSocket,
    Shutdown,
}

/// Events sent from worker threads back to UI thread
/// Optimized for high-frequency updates without blocking the UI
#[derive(Debug, Clone)]
pub enum AppEvent {
    // Data update events
    PositionsUpdated(Vec<Position>),
    OrdersUpdated(Vec<Order>),
    UserProfileUpdated(UserProfile),
    InstrumentsUpdated(Vec<Instrument>),

    // Real-time market data events (high frequency)
    TickUpdate {
        instrument_token: u32,
        last_price: f64,
        volume: u64,
        timestamp: DateTime<Utc>,
    },

    // WebSocket connection events
    WebSocketConnected,
    WebSocketDisconnected,
    WebSocketReconnecting {
        attempt: u32,
    },

    // Trading events
    OrderPlaced {
        order_id: String,
    },
    OrderModified {
        order_id: String,
    },
    OrderCancelled {
        order_id: String,
    },
    OrderFilled {
        order_id: String,
        fill_price: f64,
        fill_quantity: i32,
    },

    // System events
    Notification {
        level: LogLevel,
        message: String,
        module: Option<String>,
    },
    Error {
        error: String,
        module: Option<String>,
    },
}

/// UI input state for form fields and filters
#[derive(Debug, Default, Clone)]
pub struct UiInputState {
    // Order placement fields
    pub order_symbol_input: String,
    pub order_quantity_input: String,
    pub order_price_input: String,

    // Filters
    pub position_filter: String,
    pub order_filter: String,
    pub log_filter: String,

    // UI state
    pub show_order_dialog: bool,
    pub selected_order_type: OrderType,
    pub selected_transaction_type: TransactionType,
    pub selected_product_type: ProductType,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum OrderType {
    #[default]
    Market,
    Limit,
    StopLoss,
    StopLossMarket,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TransactionType {
    #[default]
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ProductType {
    #[default]
    CNC, // Cash and Carry
    MIS,  // Margin Intraday Squareoff
    NRML, // Normal
}

/// Main application state using high-performance concurrent data structures
/// All collections use lock-free designs for ultra-low latency access
pub struct AppState {
    // Configuration
    pub config: Config,

    // Authentication state
    pub auth_state: Arc<RwLock<AuthState>>,

    // Trading data - using DashMap for lock-free concurrent access
    pub positions: Arc<DashMap<u32, Position>>, // keyed by instrument_token
    pub orders: Arc<DashMap<String, Order>>,    // keyed by order_id
    pub instruments: Arc<DashMap<u32, Instrument>>, // keyed by instrument_token

    // User profile
    pub user_profile: Arc<RwLock<Option<UserProfile>>>,

    // Real-time data
    pub tick_data: Arc<DashMap<u32, TickData>>, // keyed by instrument_token

    // Application logs with reader-writer lock for batch operations
    pub logs: Arc<RwLock<Vec<LogEntry>>>,

    // UI state
    pub ui_input: UiInputState,

    // Communication channels
    pub command_sender: Sender<Command>,
    pub event_receiver: Receiver<AppEvent>,

    // Performance metrics
    pub metrics: Arc<RwLock<PerformanceMetrics>>,
}

/// Performance metrics for monitoring system health
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub ticks_processed: u64,
    pub orders_processed: u64,
    pub websocket_reconnections: u32,
    pub last_tick_timestamp: Option<DateTime<Utc>>,
    pub average_tick_latency_ms: f64,
}

impl AppState {
    /// Create new application state with initialized channels and data structures
    pub fn new(config: Config) -> (Self, Receiver<Command>) {
        let (command_sender, command_receiver) = crossbeam_channel::unbounded();
        let (_event_sender, event_receiver) = crossbeam_channel::unbounded();

        // Initialize with a mock logged-in state for personal trading
        let initial_auth_state = AuthState::LoggedIn {
            access_token: "mock_token".to_string(),
            user_name: "Personal Trading".to_string(),
            user_profile: None,
        };

        let state = Self {
            config,
            auth_state: Arc::new(RwLock::new(initial_auth_state)),
            positions: Arc::new(DashMap::with_capacity(1000)),
            orders: Arc::new(DashMap::with_capacity(10000)),
            instruments: Arc::new(DashMap::with_capacity(50000)),
            user_profile: Arc::new(RwLock::new(None)),
            tick_data: Arc::new(DashMap::with_capacity(1000)),
            logs: Arc::new(RwLock::new(Vec::with_capacity(10000))),
            ui_input: UiInputState::default(),
            command_sender,
            event_receiver,
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        };

        (state, command_receiver)
    }

    /// Add log entry with automatic timestamping
    pub fn add_log(&self, level: LogLevel, message: String, module: Option<String>) {
        let log_entry = LogEntry::new(level, message, module);
        let mut logs = self.logs.write();

        // Keep only the last 10000 log entries for memory efficiency
        if logs.len() >= 10000 {
            logs.drain(0..1000);
        }

        logs.push(log_entry);
    }

    /// Update position with new tick data - optimized for high frequency updates
    pub fn update_position_price(&self, instrument_token: u32, last_price: f64) {
        if let Some(mut position) = self.positions.get_mut(&instrument_token) {
            position.update_last_price(last_price);
        }
    }

    /// Calculate total PnL across all positions
    pub fn calculate_total_pnl(&self) -> PnlData {
        let mut realized = 0.0;
        let mut unrealized = 0.0;

        for entry in self.positions.iter() {
            let position = entry.value();
            realized += position.realized_pnl;
            unrealized += position.unrealized_pnl;
        }

        PnlData {
            realized,
            unrealized,
            total: realized + unrealized,
            day_pnl: unrealized, // Simplified - in real implementation, track day-specific PnL
            day_realized: 0.0,
            day_unrealized: unrealized,
        }
    }

    /// Get filtered orders based on tradingsymbol
    pub fn get_filtered_orders(&self, filter: &str) -> Vec<Order> {
        if filter.is_empty() {
            self.orders
                .iter()
                .map(|entry| entry.value().clone())
                .collect()
        } else {
            self.orders
                .iter()
                .filter(|entry| {
                    entry
                        .value()
                        .tradingsymbol
                        .to_lowercase()
                        .contains(&filter.to_lowercase())
                })
                .map(|entry| entry.value().clone())
                .collect()
        }
    }

    /// Send command to worker threads
    pub fn send_command(&self, command: Command) {
        if let Err(e) = self.command_sender.send(command) {
            self.add_log(
                LogLevel::Error,
                format!("Failed to send command: {}", e),
                Some("state".to_string()),
            );
        }
    }

    /// Process all pending events from worker threads
    pub fn process_events(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(event);
        }
    }

    /// Handle individual events from worker threads
    fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::PositionsUpdated(positions) => {
                self.positions.clear();
                for position in positions {
                    self.positions.insert(position.instrument_token, position);
                }

                self.add_log(
                    LogLevel::Info,
                    format!("Updated {} positions", self.positions.len()),
                    Some("positions".to_string()),
                );
            }

            AppEvent::OrdersUpdated(orders) => {
                // Update orders, preserving existing ones not in the update
                for order in orders {
                    self.orders.insert(order.order_id.clone(), order);
                }

                self.add_log(
                    LogLevel::Info,
                    format!("Updated orders, total: {}", self.orders.len()),
                    Some("orders".to_string()),
                );
            }

            AppEvent::TickUpdate {
                instrument_token,
                last_price,
                volume,
                timestamp,
            } => {
                // Update position prices for real-time PnL calculation
                self.update_position_price(instrument_token, last_price);

                // Update tick data
                if let Some(mut tick_data) = self.tick_data.get_mut(&instrument_token) {
                    tick_data.last_price = last_price;
                    tick_data.volume = volume;
                    tick_data.timestamp_nanos = timestamp.timestamp_nanos();
                } else {
                    // Create new tick data entry
                    let tick_data = TickData {
                        instrument_token,
                        last_price,
                        last_quantity: 0,
                        average_price: last_price,
                        volume,
                        buy_quantity: 0,
                        sell_quantity: 0,
                        ohlc: OHLC {
                            open: last_price,
                            high: last_price,
                            low: last_price,
                            close: last_price,
                        },
                        timestamp_nanos: timestamp.timestamp_nanos(),
                    };
                    self.tick_data.insert(instrument_token, tick_data);
                }

                // Update metrics
                let mut metrics = self.metrics.write();
                metrics.ticks_processed += 1;
                metrics.last_tick_timestamp = Some(timestamp);
            }

            AppEvent::Notification {
                level,
                message,
                module,
            } => {
                self.add_log(level, message, module);
            }

            AppEvent::Error { error, module } => {
                self.add_log(LogLevel::Error, error, module);
            }

            // Handle other events...
            _ => {
                self.add_log(
                    LogLevel::Debug,
                    format!("Unhandled event: {:?}", event),
                    Some("state".to_string()),
                );
            }
        }
    }
}

/// Event sender handle for worker threads
/// Allows workers to send events back to the UI thread
#[derive(Clone)]
pub struct EventSender {
    sender: Sender<AppEvent>,
}

impl EventSender {
    pub fn new(sender: Sender<AppEvent>) -> Self {
        Self { sender }
    }

    pub fn send(&self, event: AppEvent) -> Result<(), crossbeam_channel::SendError<AppEvent>> {
        self.sender.send(event)
    }

    pub fn send_notification(&self, level: LogLevel, message: String, module: Option<String>) {
        let _ = self.send(AppEvent::Notification {
            level,
            message,
            module,
        });
    }

    pub fn send_error(&self, error: String, module: Option<String>) {
        let _ = self.send(AppEvent::Error { error, module });
    }
}
