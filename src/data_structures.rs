use chrono::{DateTime, Utc};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// High-performance Position struct with zero-copy deserialization for WebSocket updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "rkyv", derive(Archive, RkyvSerialize, RkyvDeserialize))]
pub struct Position {
    pub instrument_token: u32,
    pub tradingsymbol: String,
    pub exchange: String,
    pub product: String,
    pub quantity: i32,
    pub average_price: f64,
    pub last_price: f64,
    pub close_price: f64,
    pub pnl: f64, // Calculated dynamically: (last_price - average_price) * quantity
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub multiplier: f64,
    pub overnight_quantity: i32,
    pub day_quantity: i32,
}

impl Position {
    /// Calculate PnL dynamically for ultra-low latency updates
    pub fn calculate_pnl(&mut self) {
        if self.quantity != 0 {
            self.pnl = (self.last_price - self.average_price) * self.quantity as f64;
            self.unrealized_pnl = (self.last_price - self.close_price) * self.quantity as f64;
        }
    }

    /// Update last price and recalculate PnL in a single atomic operation
    pub fn update_last_price(&mut self, new_price: f64) {
        self.last_price = new_price;
        self.calculate_pnl();
    }
}

/// High-performance Order struct optimized for frequent updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "rkyv", derive(Archive, RkyvSerialize, RkyvDeserialize))]
pub struct Order {
    pub order_id: String,
    pub parent_order_id: Option<String>,
    pub exchange_order_id: String,
    pub placed_by: String,
    pub variety: String,
    pub status: OrderStatus,
    pub tradingsymbol: String,
    pub exchange: String,
    pub instrument_token: u32,
    pub transaction_type: String,
    pub order_type: String,
    pub product: String,
    pub validity: String,
    pub price: f64,
    pub quantity: i32,
    pub pending_quantity: i32,
    pub filled_quantity: i32,
    pub disclosed_quantity: i32,
    pub trigger_price: f64,
    pub average_price: f64,
    pub order_timestamp: DateTime<Utc>,
    pub exchange_timestamp: Option<DateTime<Utc>>,
    pub status_message: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "rkyv", derive(Archive, RkyvSerialize, RkyvDeserialize))]
pub enum OrderStatus {
    Open,
    Complete,
    Cancelled,
    Rejected,
    Trigger,
    Modified,
}

/// Zero-copy tick data structure for ultra-low latency WebSocket processing
#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, RkyvSerialize, RkyvDeserialize))]
pub struct TickData {
    pub instrument_token: u32,
    pub last_price: f64,
    pub last_quantity: u32,
    pub average_price: f64,
    pub volume: u64,
    pub buy_quantity: u64,
    pub sell_quantity: u64,
    pub ohlc: OHLC,
    pub timestamp_nanos: i64, // Unix timestamp in nanoseconds
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "rkyv", derive(Archive, RkyvSerialize, RkyvDeserialize))]
pub struct OHLC {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

/// Order request structure for placing new orders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub tradingsymbol: String,
    pub exchange: String,
    pub transaction_type: String, // BUY or SELL
    pub order_type: String,       // MARKET, LIMIT, SL, SL-M
    pub quantity: i32,
    pub price: Option<f64>,
    pub product: String,  // CNC, MIS, NRML
    pub validity: String, // DAY, IOC
    pub disclosed_quantity: Option<i32>,
    pub trigger_price: Option<f64>,
    pub squareoff: Option<f64>,
    pub stoploss: Option<f64>,
    pub trailing_stoploss: Option<f64>,
    pub tag: Option<String>,
}

/// PnL data structure for performance analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnlData {
    pub realized: f64,
    pub unrealized: f64,
    pub total: f64,
    pub day_pnl: f64,
    pub day_realized: f64,
    pub day_unrealized: f64,
}

/// Log levels for the trading application
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "rkyv", derive(Archive, RkyvSerialize, RkyvDeserialize))]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

/// Log entry structure for application logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub module: Option<String>,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: String, module: Option<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            message,
            module,
        }
    }
}

/// User profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub user_name: String,
    pub email: String,
    pub user_type: String,
    pub broker: String,
    pub exchanges: Vec<String>,
    pub products: Vec<String>,
    pub order_types: Vec<String>,
}

/// Instrument master data for symbol lookup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub instrument_token: u32,
    pub exchange_token: u32,
    pub tradingsymbol: String,
    pub name: String,
    pub last_price: f64,
    pub expiry: Option<String>,
    pub strike: Option<f64>,
    pub tick_size: f64,
    pub lot_size: u32,
    pub instrument_type: String,
    pub segment: String,
    pub exchange: String,
}
