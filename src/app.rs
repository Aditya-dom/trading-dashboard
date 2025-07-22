use crate::data_structures::LogLevel;
use crate::state::{AppState, AuthState, Config, EventSender};
use crate::ui;
use crate::workers::{ApiHandler, WebSocketHandler};
use crossbeam_channel::Receiver;
use std::sync::Arc;

/// Main trading application implementing eframe::App
/// Designed for ultra-low latency UI updates and responsive user interaction
pub struct TradingApp {
    app_state: AppState,
    current_view: AppView,
    // Worker handles for cleanup
    _api_handler: tokio::task::JoinHandle<()>,
    _websocket_handler: tokio::task::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppView {
    Overview,
    Positions,
    Orders,
    PnL,
    Logs,
}

impl TradingApp {
    /// Create new trading application with all workers and communication channels
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Load configuration
        let config = Config::load().expect("Failed to load configuration");

        // Initialize application state and channels
        let (mut app_state, command_receiver) = AppState::new(config.clone());

        // Create event sender for workers
        let (event_sender_tx, event_receiver_rx) = crossbeam_channel::unbounded();
        let event_sender = EventSender::new(event_sender_tx);

        // Update app state with event receiver
        app_state.event_receiver = event_receiver_rx;

        // Start API handler worker
        let api_handler = ApiHandler::new(config.clone(), event_sender.clone());
        let command_receiver_clone = command_receiver.clone();
        let api_handler_task = tokio::spawn(async move {
            let mut handler = api_handler;
            handler.run(command_receiver_clone).await;
        });

        // Start WebSocket handler worker
        let websocket_handler = WebSocketHandler::new(config.clone(), event_sender.clone());
        let command_receiver_clone = command_receiver.clone();
        let websocket_handler_task = tokio::spawn(async move {
            let mut handler = websocket_handler;
            handler.run(command_receiver_clone).await;
        });

        app_state.add_log(
            LogLevel::Info,
            "Trading application initialized".to_string(),
            Some("app".to_string()),
        );

        Self {
            app_state,
            current_view: AppView::Overview,
            _api_handler: api_handler_task,
            _websocket_handler: websocket_handler_task,
        }
    }

    /// Render main navigation tabs
    fn render_navigation(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.current_view, AppView::Overview, "📊 Overview");
            ui.selectable_value(&mut self.current_view, AppView::Positions, "💼 Positions");
            ui.selectable_value(&mut self.current_view, AppView::Orders, "📋 Orders");
            ui.selectable_value(&mut self.current_view, AppView::PnL, "💰 P&L");
            ui.selectable_value(&mut self.current_view, AppView::Logs, "📝 Logs");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Show personal trading indicator
                ui.label("👤 Personal Trading");
            });
        });
    }

    /// Render main content area based on current view
    fn render_content(&mut self, ui: &mut egui::Ui) {
        match self.current_view {
            AppView::Overview => {
                ui::render_overview(ui, &mut self.app_state);
            }
            AppView::Positions => {
                ui::render_positions(ui, &mut self.app_state);
            }
            AppView::Orders => {
                ui::render_orders(ui, &mut self.app_state);
            }
            AppView::PnL => {
                ui::render_pnl(ui, &mut self.app_state);
            }
            AppView::Logs => {
                ui::render_logs(ui, &mut self.app_state);
            }
        }
    }

    /// Render status bar with connection info and metrics
    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Connection status
            let metrics = self.app_state.metrics.read();

            if let Some(last_tick) = metrics.last_tick_timestamp {
                let elapsed = chrono::Utc::now().signed_duration_since(last_tick);
                if elapsed.num_seconds() < 5 {
                    ui.colored_label(egui::Color32::GREEN, "🟢 Live");
                } else {
                    ui.colored_label(egui::Color32::YELLOW, "🟡 Delayed");
                }
            } else {
                ui.colored_label(egui::Color32::RED, "🔴 Disconnected");
            }

            ui.separator();

            // Performance metrics
            ui.label(format!("Ticks: {}", metrics.ticks_processed));
            ui.label(format!("Orders: {}", metrics.orders_processed));

            if metrics.average_tick_latency_ms > 0.0 {
                ui.label(format!("Latency: {:.1}ms", metrics.average_tick_latency_ms));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Current time
                let now = chrono::Local::now();
                ui.label(format!("🕐 {}", now.format("%H:%M:%S")));
            });
        });
    }
}

impl eframe::App for TradingApp {
    /// Main update loop - processes events and renders UI
    /// Optimized for 60+ FPS with minimal allocations
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process all pending events from worker threads
        self.app_state.process_events();

        // Main application UI - always show since we're bypassing authentication
        egui::TopBottomPanel::top("nav_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            self.render_navigation(ui);
            ui.add_space(5.0);
        });

        egui::TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            ui.add_space(3.0);
            self.render_status_bar(ui);
            ui.add_space(3.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_content(ui);
        });

        // Request repaint for real-time updates
        ctx.request_repaint();
    }

    /// Handle application shutdown
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.app_state.add_log(
            LogLevel::Info,
            "Application shutting down".to_string(),
            Some("app".to_string()),
        );

        // Send shutdown command to workers
        self.app_state.send_command(crate::state::Command::Shutdown);
    }
}
