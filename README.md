# Rust Trading Dashboard

A high-performance, real-time trading dashboard built in Rust using egui, designed for ultra-low latency operations with the Zerodha Kite API.

## Features

### Core Architecture
- **Multi-threaded, Event-driven Design**: UI thread remains non-blocking with dedicated worker threads
- **High-Performance Concurrency**: Uses `crossbeam`, `dashmap`, and `parking_lot` for lock-free operations
- **Zero-Copy Deserialization**: `rkyv` for WebSocket tick data processing
- **Real-time Updates**: Live P&L calculations and position tracking

### Trading Capabilities
- **Real-time Market Data**: WebSocket integration for live price feeds
- **Position Management**: Live P&L tracking with real-time price updates
- **Order Management**: Place, modify, and cancel orders with comprehensive tracking
- **Authentication**: Secure Zerodha OAuth integration
- **Multi-exchange Support**: NSE, BSE, and other supported exchanges


## Architecture

### Technology Stack
- **GUI Framework**: `egui` + `eframe` for immediate mode rendering
- **Async Runtime**: `tokio` for high-performance async operations
- **Concurrency**: `crossbeam-channel` for lock-free message passing
- **Collections**: `dashmap` for concurrent hash maps
- **Serialization**: `rkyv` (zero-copy) + `serde` (REST APIs)
- **HTTP Client**: `reqwest` with connection pooling
- **WebSocket**: `tokio-tungstenite` for real-time data
- **Configuration**: `figment` with TOML support

### Design Patterns
- **Event-driven Architecture**: Commands flow UI→Workers, Events flow Workers→UI
- **Shared State**: `Arc<DashMap>` for lock-free concurrent access
- **Worker Isolation**: Separate threads for API and WebSocket operations
- **Error Handling**: Comprehensive error propagation and user feedback

## 📁 Project Structure

```
trading_dashboard/
├── Cargo.toml              # Dependencies and build configuration
├── config.toml             # API keys and application settings
└── src/
    ├── main.rs             # Application entry point and UI styling
    ├── app.rs              # Main TradingApp struct and eframe integration
    ├── state.rs            # Application state, commands, and events
    ├── data_structures.rs  # Core trading data types with optimizations
    │
    ├── api/
    │   └── zerodha_client.rs # Zerodha REST API client implementation
    │
    ├── workers/
    │   ├── api_handler.rs    # REST API operations worker
    │   └── websocket_handler.rs # Real-time WebSocket data worker
    │
    └── ui/
        ├── components/       # Reusable UI components
        ├── login.rs         # Authentication interface
        ├── overview.rs      # Dashboard overview
        ├── positions.rs     # Position management interface
        ├── orders.rs        # Order management interface
        ├── pnl.rs          # P&L analytics
        └── logs.rs         # Application logs viewer
```

## Configuration

### API Configuration
Edit `config.toml`:

```toml
[zerodha]
api_key = "your_zerodha_api_key"
api_secret = "your_zerodha_api_secret"
redirect_url = "http://localhost:8080"

[app]
log_level = "info"
websocket_reconnect_delay_ms = 1000
max_reconnect_attempts = 10
tick_buffer_size = 1000
```

### Zerodha API Setup
1. Create a Kite Connect app at [developers.kite.trade](https://developers.kite.trade)
2. Get your `api_key` and `api_secret`
3. Set redirect URL to `http://localhost:8080` (or your preferred URL)
4. Update `config.toml` with your credentials

### Installation & Running

```bash
# Clone the repository
git clone <repository_url>
cd trading_dashboard

# Update config.toml with your API credentials
nano config.toml

# Build and run (development)
cargo run

# Build optimized release version
cargo build --release
./target/release/trading_dashboard
```

### First-time Setup
1. **Start the application**: The login screen will appear
2. **Click "Login with Zerodha"**: This generates your login URL
3. **Browser authentication**: Login via your browser, copy the request_token from the redirect URL
4. **Complete authentication**: Paste the request_token back into the app
5. **Start trading**: Access positions, orders, and real-time data

## Performance Optimizations

### Ultra-Low Latency Features
- **Lock-free Data Structures**: `DashMap` for concurrent access without blocking
- **Zero-copy Deserialization**: `rkyv` for WebSocket tick processing
- **Connection Pooling**: Persistent HTTP connections for API calls
- **Batched Updates**: UI updates are batched to maintain 60+ FPS
- **Memory Efficient**: Pre-allocated buffers and minimal allocations

### Concurrent Design
- **UI Thread**: Pure rendering, never blocks on I/O
- **API Worker**: Handles all REST API communication
- **WebSocket Worker**: Real-time market data with auto-reconnection
- **Event System**: `crossbeam-channel` for high-throughput message passing

## Key Features

### Real-time Trading
- **Live Position Tracking**: P&L updates with every price tick
- **Order Management**: Place, modify, cancel orders with real-time status
- **Market Data**: Subscribe to instrument price feeds
- **Performance Metrics**: Latency monitoring and connection status

### Professional UI
- **Dark Theme**: Optimized for trading environments
- **Data Tables**: High-density information display
- **Color Coding**: Green/red P&L, status indicators
- **Responsive**: Real-time updates without UI blocking
- **Filtering**: Quick search and filter capabilities

### Risk Management
- **Real-time P&L**: Instant profit/loss calculations
- **Position Overview**: Quick portfolio assessment
- **Order Status**: Real-time order execution tracking
- **Error Handling**: Comprehensive error reporting and recovery

### Performance Considerations
- **Memory Management**: Minimal allocations in hot paths
- **Async Operations**: Non-blocking I/O throughout
- **Efficient Data Structures**: Optimized for trading data patterns
- **Resource Management**: Proper cleanup and resource handling

## Trading Workflow

1. **Authentication**: Secure OAuth flow with Zerodha
2. **Data Subscription**: Subscribe to relevant instrument price feeds
3. **Position Monitoring**: Real-time P&L tracking
4. **Order Execution**: Quick order placement and management
5. **Performance Analysis**: P&L analytics and trading metrics
