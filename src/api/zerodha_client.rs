use crate::data_structures::*;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// High-performance Zerodha API client optimized for low-latency trading
/// Uses connection pooling and async I/O for maximum throughput
pub struct ZerodhaClient {
    client: Client,
    api_key: String,
    api_secret: String,
    access_token: Option<String>,
    base_url: String,
}

/// API response wrapper for Zerodha REST responses
#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub data: Option<T>,
    pub message: Option<String>,
    pub error_type: Option<String>,
}

/// Session data returned after successful authentication
#[derive(Debug, Deserialize)]
pub struct SessionData {
    pub access_token: String,
    pub refresh_token: String,
    pub user_id: String,
    pub user_name: String,
    pub user_shortname: String,
    pub email: String,
    pub user_type: String,
    pub broker: String,
    pub exchanges: Vec<String>,
    pub products: Vec<String>,
    pub order_types: Vec<String>,
}

/// Order response from Zerodha API
#[derive(Debug, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
}

impl ZerodhaClient {
    /// Create new Zerodha client with optimized HTTP client configuration
    pub fn new(api_key: String, api_secret: String) -> Self {
        let client = Client::builder()
            .tcp_keepalive(std::time::Duration::from_secs(30))
            .timeout(std::time::Duration::from_secs(10))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            api_secret,
            access_token: None,
            base_url: "https://api.kite.trade".to_string(),
        }
    }

    /// Set the access token for API calls (for personal trading)
    pub fn set_access_token(&mut self, access_token: String) {
        self.access_token = Some(access_token);
    }

    /// Generate login URL for Zerodha OAuth flow
    pub fn generate_login_url(&self, _redirect_url: &str) -> Result<String> {
        let mut url = Url::parse("https://kite.trade/connect/login")?;
        url.query_pairs_mut()
            .append_pair("api_key", &self.api_key)
            .append_pair("v", "3");

        Ok(url.to_string())
    }

    /// Exchange request token for access token - critical for authentication flow
    pub async fn generate_session(
        &mut self,
        request_token: &str,
        checksum: &str,
    ) -> Result<SessionData> {
        let url = format!("{}/session/token", self.base_url);

        let mut params = HashMap::new();
        params.insert("api_key", self.api_key.as_str());
        params.insert("request_token", request_token);
        params.insert("checksum", checksum);

        let response = self
            .client
            .post(&url)
            .form(&params)
            .send()
            .await
            .context("Failed to send session token request")?;

        let api_response: ApiResponse<SessionData> = response
            .json()
            .await
            .context("Failed to parse session response")?;

        match api_response.status.as_str() {
            "success" => {
                if let Some(session_data) = api_response.data {
                    self.access_token = Some(session_data.access_token.clone());
                    Ok(session_data)
                } else {
                    anyhow::bail!("Session data not found in response")
                }
            }
            _ => {
                let error_msg = api_response
                    .message
                    .unwrap_or_else(|| "Authentication failed".to_string());
                anyhow::bail!("Authentication error: {}", error_msg)
            }
        }
    }

    /// Generate checksum for session token request using HMAC-SHA256
    pub fn generate_checksum(&self, request_token: &str) -> String {
        use sha2::{Digest, Sha256};

        let message = format!("{}{}{}", self.api_key, request_token, self.api_secret);
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        let result = hasher.finalize();

        hex::encode(result)
    }

    /// Fetch user positions with error handling and retries
    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        let access_token = self
            .access_token
            .as_ref()
            .context("Access token not available")?;

        let url = format!("{}/portfolio/positions", self.base_url);

        let response = self
            .client
            .get(&url)
            .header(
                "Authorization",
                format!("token {}:{}", self.api_key, access_token),
            )
            .send()
            .await
            .context("Failed to fetch positions")?;

        let api_response: ApiResponse<PositionsResponse> = response
            .json()
            .await
            .context("Failed to parse positions response")?;

        match api_response.status.as_str() {
            "success" => {
                if let Some(positions_data) = api_response.data {
                    // Convert API positions to our Position struct
                    let mut positions = Vec::new();
                    for net_position in positions_data.net {
                        let mut position = Position {
                            instrument_token: net_position.instrument_token,
                            tradingsymbol: net_position.tradingsymbol,
                            exchange: net_position.exchange,
                            product: net_position.product,
                            quantity: net_position.quantity,
                            average_price: net_position.average_price,
                            last_price: net_position.last_price,
                            close_price: net_position.close_price,
                            pnl: 0.0, // Will be calculated
                            unrealized_pnl: net_position.unrealised,
                            realized_pnl: net_position.realised,
                            multiplier: net_position.multiplier,
                            overnight_quantity: net_position.overnight_quantity.unwrap_or(0),
                            day_quantity: net_position.day_quantity.unwrap_or(0),
                        };
                        position.calculate_pnl();
                        positions.push(position);
                    }
                    Ok(positions)
                } else {
                    Ok(Vec::new())
                }
            }
            _ => {
                let error_msg = api_response
                    .message
                    .unwrap_or_else(|| "Failed to fetch positions".to_string());
                anyhow::bail!("API error: {}", error_msg)
            }
        }
    }

    /// Fetch user orders with comprehensive error handling
    pub async fn get_orders(&self) -> Result<Vec<Order>> {
        let access_token = self
            .access_token
            .as_ref()
            .context("Access token not available")?;

        let url = format!("{}/orders", self.base_url);

        let response = self
            .client
            .get(&url)
            .header(
                "Authorization",
                format!("token {}:{}", self.api_key, access_token),
            )
            .send()
            .await
            .context("Failed to fetch orders")?;

        let api_response: ApiResponse<Vec<ApiOrder>> = response
            .json()
            .await
            .context("Failed to parse orders response")?;

        match api_response.status.as_str() {
            "success" => {
                if let Some(api_orders) = api_response.data {
                    let orders = api_orders
                        .into_iter()
                        .map(|api_order| self.convert_api_order(api_order))
                        .collect();
                    Ok(orders)
                } else {
                    Ok(Vec::new())
                }
            }
            _ => {
                let error_msg = api_response
                    .message
                    .unwrap_or_else(|| "Failed to fetch orders".to_string());
                anyhow::bail!("API error: {}", error_msg)
            }
        }
    }

    /// Place a new order with comprehensive validation
    pub async fn place_order(&self, order_request: &OrderRequest) -> Result<String> {
        let access_token = self
            .access_token
            .as_ref()
            .context("Access token not available")?;

        let url = format!("{}/orders/regular", self.base_url);

        let mut params = HashMap::new();
        params.insert("tradingsymbol", order_request.tradingsymbol.as_str());
        params.insert("exchange", order_request.exchange.as_str());
        params.insert("transaction_type", order_request.transaction_type.as_str());
        params.insert("order_type", order_request.order_type.as_str());

        let quantity_str = order_request.quantity.to_string();
        params.insert("quantity", quantity_str.as_str());

        params.insert("product", order_request.product.as_str());
        params.insert("validity", order_request.validity.as_str());

        let price_str;
        if let Some(price) = order_request.price {
            price_str = price.to_string();
            params.insert("price", price_str.as_str());
        }

        let trigger_price_str;
        if let Some(trigger_price) = order_request.trigger_price {
            trigger_price_str = trigger_price.to_string();
            params.insert("trigger_price", trigger_price_str.as_str());
        }

        let disclosed_quantity_str;
        if let Some(disclosed_quantity) = order_request.disclosed_quantity {
            disclosed_quantity_str = disclosed_quantity.to_string();
            params.insert("disclosed_quantity", disclosed_quantity_str.as_str());
        }

        if let Some(tag) = &order_request.tag {
            params.insert("tag", tag.as_str());
        }

        let response = self
            .client
            .post(&url)
            .header(
                "Authorization",
                format!("token {}:{}", self.api_key, access_token),
            )
            .form(&params)
            .send()
            .await
            .context("Failed to place order")?;

        let api_response: ApiResponse<OrderResponse> = response
            .json()
            .await
            .context("Failed to parse order response")?;

        match api_response.status.as_str() {
            "success" => {
                if let Some(order_data) = api_response.data {
                    Ok(order_data.order_id)
                } else {
                    anyhow::bail!("Order ID not found in response")
                }
            }
            _ => {
                let error_msg = api_response
                    .message
                    .unwrap_or_else(|| "Failed to place order".to_string());
                anyhow::bail!("Order placement error: {}", error_msg)
            }
        }
    }

    /// Cancel an existing order
    pub async fn cancel_order(&self, order_id: &str, variety: &str) -> Result<String> {
        let access_token = self
            .access_token
            .as_ref()
            .context("Access token not available")?;

        let url = format!("{}/orders/{}/{}", self.base_url, variety, order_id);

        let response = self
            .client
            .delete(&url)
            .header(
                "Authorization",
                format!("token {}:{}", self.api_key, access_token),
            )
            .send()
            .await
            .context("Failed to cancel order")?;

        let api_response: ApiResponse<OrderResponse> = response
            .json()
            .await
            .context("Failed to parse cancel response")?;

        match api_response.status.as_str() {
            "success" => {
                if let Some(order_data) = api_response.data {
                    Ok(order_data.order_id)
                } else {
                    anyhow::bail!("Order ID not found in response")
                }
            }
            _ => {
                let error_msg = api_response
                    .message
                    .unwrap_or_else(|| "Failed to cancel order".to_string());
                anyhow::bail!("Order cancellation error: {}", error_msg)
            }
        }
    }

    /// Fetch instrument master data for symbol lookup
    pub async fn get_instruments(&self, exchange: &str) -> Result<Vec<Instrument>> {
        let url = format!("{}/instruments/{}", self.base_url, exchange);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch instruments")?;

        let csv_data = response
            .text()
            .await
            .context("Failed to get instruments CSV")?;

        // Parse CSV data into Instrument structs
        self.parse_instruments_csv(&csv_data)
    }

    /// Convert API order format to our Order struct
    fn convert_api_order(&self, api_order: ApiOrder) -> Order {
        let status = match api_order.status.as_str() {
            "OPEN" => OrderStatus::Open,
            "COMPLETE" => OrderStatus::Complete,
            "CANCELLED" => OrderStatus::Cancelled,
            "REJECTED" => OrderStatus::Rejected,
            "TRIGGER PENDING" => OrderStatus::Trigger,
            "MODIFIED" => OrderStatus::Modified,
            _ => OrderStatus::Open,
        };

        Order {
            order_id: api_order.order_id,
            parent_order_id: api_order.parent_order_id,
            exchange_order_id: api_order.exchange_order_id.unwrap_or_default(),
            placed_by: api_order.placed_by,
            variety: api_order.variety,
            status,
            tradingsymbol: api_order.tradingsymbol,
            exchange: api_order.exchange,
            instrument_token: api_order.instrument_token,
            transaction_type: api_order.transaction_type,
            order_type: api_order.order_type,
            product: api_order.product,
            validity: api_order.validity,
            price: api_order.price,
            quantity: api_order.quantity,
            pending_quantity: api_order.pending_quantity,
            filled_quantity: api_order.filled_quantity,
            disclosed_quantity: api_order.disclosed_quantity,
            trigger_price: api_order.trigger_price,
            average_price: api_order.average_price,
            order_timestamp: api_order.order_timestamp,
            exchange_timestamp: api_order.exchange_timestamp,
            status_message: api_order.status_message,
            tag: api_order.tag,
        }
    }

    /// Parse instruments CSV data - optimized for large datasets
    fn parse_instruments_csv(&self, csv_data: &str) -> Result<Vec<Instrument>> {
        let mut instruments = Vec::new();
        let mut lines = csv_data.lines();

        // Skip header
        lines.next();

        for line in lines {
            let fields: Vec<&str> = line.split(',').collect();
            if fields.len() >= 12 {
                let instrument = Instrument {
                    instrument_token: fields[0].parse().unwrap_or(0),
                    exchange_token: fields[1].parse().unwrap_or(0),
                    tradingsymbol: fields[2].to_string(),
                    name: fields[3].to_string(),
                    last_price: fields[4].parse().unwrap_or(0.0),
                    expiry: if fields[5].is_empty() {
                        None
                    } else {
                        Some(fields[5].to_string())
                    },
                    strike: if fields[6].is_empty() {
                        None
                    } else {
                        fields[6].parse().ok()
                    },
                    tick_size: fields[7].parse().unwrap_or(0.01),
                    lot_size: fields[8].parse().unwrap_or(1),
                    instrument_type: fields[9].to_string(),
                    segment: fields[10].to_string(),
                    exchange: fields[11].to_string(),
                };
                instruments.push(instrument);
            }
        }

        Ok(instruments)
    }
}

/// API response structures for positions
#[derive(Debug, Deserialize)]
struct PositionsResponse {
    net: Vec<ApiPosition>,
    day: Vec<ApiPosition>,
}

#[derive(Debug, Deserialize)]
struct ApiPosition {
    tradingsymbol: String,
    exchange: String,
    instrument_token: u32,
    product: String,
    quantity: i32,
    overnight_quantity: Option<i32>,
    multiplier: f64,
    average_price: f64,
    close_price: f64,
    last_price: f64,
    value: f64,
    pnl: f64,
    m2m: f64,
    unrealised: f64,
    realised: f64,
    day_quantity: Option<i32>,
}

/// API response structure for orders
#[derive(Debug, Deserialize)]
struct ApiOrder {
    order_id: String,
    parent_order_id: Option<String>,
    exchange_order_id: Option<String>,
    placed_by: String,
    variety: String,
    status: String,
    tradingsymbol: String,
    exchange: String,
    instrument_token: u32,
    transaction_type: String,
    order_type: String,
    product: String,
    validity: String,
    price: f64,
    quantity: i32,
    pending_quantity: i32,
    filled_quantity: i32,
    disclosed_quantity: i32,
    trigger_price: f64,
    average_price: f64,
    order_timestamp: chrono::DateTime<chrono::Utc>,
    exchange_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    status_message: Option<String>,
    tag: Option<String>,
}
