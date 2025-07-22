mod api;
mod app;
mod data_structures;
mod state;
mod ui;
mod workers;

use app::TradingApp;
use eframe::egui;

/// Main entry point for the professional-grade Rust trading dashboard
/// Optimized for ultra-low latency trading operations
#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    // Initialize logging
    env_logger::init();

    // Configure eframe options for optimal performance
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Professional Trading Dashboard")
            .with_min_inner_size([800.0, 600.0]),

        // Enable hardware acceleration for better performance
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,

        // Enable multi-sampling for smoother rendering
        multisampling: 4,

        // Configure for trading application requirements
        persist_window: true,
        centered: true,

        ..Default::default()
    };

    log::info!("Starting Trading Dashboard...");

    // Run the application
    eframe::run_native(
        "Trading Dashboard",
        options,
        Box::new(|cc| {
            // Configure egui style for professional appearance
            configure_ui_style(&cc.egui_ctx);

            Ok(Box::new(TradingApp::new(cc)))
        }),
    )
}

/// Configure the UI style for a professional trading interface
fn configure_ui_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Dark theme optimized for trading
    style.visuals = egui::Visuals::dark();

    // Spacing and sizing for dense information display
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.menu_margin = egui::Margin::same(8);
    style.spacing.indent = 16.0;

    // Colors optimized for trading data
    style.visuals.extreme_bg_color = egui::Color32::from_gray(16); // Very dark background
    style.visuals.panel_fill = egui::Color32::from_gray(24); // Panel background
    style.visuals.window_fill = egui::Color32::from_gray(32); // Window background

    // Text colors for readability
    style.visuals.override_text_color = Some(egui::Color32::from_gray(240));

    // Accent colors for interactive elements
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(59, 130, 246);
    style.visuals.hyperlink_color = egui::Color32::from_rgb(96, 165, 250);

    // Grid and stroke colors
    style.visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_gray(64));
    style.visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_gray(64));

    // Interactive widget styling
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_gray(48);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_gray(64);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_gray(80);

    // Fonts for better readability of numbers and data
    let mut fonts = egui::FontDefinitions::default();

    // Use built-in monospace font for numerical data
    // In a production app, you could load custom fonts here

    ctx.set_fonts(fonts);
    ctx.set_style(style);

    log::info!("UI style configured for trading dashboard");
}
