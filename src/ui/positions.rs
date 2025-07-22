use crate::data_structures::*;
use crate::state::{AppState, Command};
use crate::ui::components::{danger_button, primary_button, success_button};
use egui::{Color32, RichText, ScrollArea, Ui};

/// Render positions table with real-time P&L updates
/// Optimized for high-frequency price updates without UI stuttering
pub fn render_positions(ui: &mut Ui, app_state: &mut AppState) {
    ui.vertical(|ui| {
        // Header with refresh button
        ui.horizontal(|ui| {
            ui.label(RichText::new("Positions").size(24.0).strong());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if primary_button("🔄 Refresh")
                    .size(egui::Vec2::new(100.0, 30.0))
                    .ui(ui)
                    .clicked()
                {
                    app_state.send_command(Command::FetchPositions);

                    app_state.add_log(
                        LogLevel::Info,
                        "Refreshing positions...".to_string(),
                        Some("positions".to_string()),
                    );
                }
            });
        });

        ui.add_space(10.0);

        // Summary cards
        render_positions_summary_cards(ui, app_state);

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // Filter input
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut app_state.ui_input.position_filter);

            if ui.button("Clear").clicked() {
                app_state.ui_input.position_filter.clear();
            }
        });

        ui.add_space(10.0);

        // Positions table
        if app_state.positions.is_empty() {
            render_empty_positions(ui);
        } else {
            render_positions_table(ui, app_state);
        }
    });
}

/// Render summary cards with aggregated position data
fn render_positions_summary_cards(ui: &mut Ui, app_state: &AppState) {
    let pnl_data = app_state.calculate_total_pnl();

    ui.horizontal(|ui| {
        // Total P&L card
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Total P&L").strong());
                let color = if pnl_data.total >= 0.0 {
                    Color32::from_rgb(34, 197, 94) // Green
                } else {
                    Color32::from_rgb(239, 68, 68) // Red
                };
                let prefix = if pnl_data.total >= 0.0 { "+" } else { "" };
                ui.label(
                    RichText::new(format!("{}₹{:.2}", prefix, pnl_data.total))
                        .size(20.0)
                        .color(color),
                );
            });
        });

        ui.add_space(10.0);

        // Realized P&L card
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Realized").strong());
                let color = if pnl_data.realized >= 0.0 {
                    Color32::from_rgb(34, 197, 94)
                } else {
                    Color32::from_rgb(239, 68, 68)
                };
                let prefix = if pnl_data.realized >= 0.0 { "+" } else { "" };
                ui.label(
                    RichText::new(format!("{}₹{:.2}", prefix, pnl_data.realized))
                        .size(16.0)
                        .color(color),
                );
            });
        });

        ui.add_space(10.0);

        // Unrealized P&L card
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Unrealized").strong());
                let color = if pnl_data.unrealized >= 0.0 {
                    Color32::from_rgb(34, 197, 94)
                } else {
                    Color32::from_rgb(239, 68, 68)
                };
                let prefix = if pnl_data.unrealized >= 0.0 { "+" } else { "" };
                ui.label(
                    RichText::new(format!("{}₹{:.2}", prefix, pnl_data.unrealized))
                        .size(16.0)
                        .color(color),
                );
            });
        });

        ui.add_space(10.0);

        // Position count card
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Positions").strong());
                ui.label(
                    RichText::new(format!("{}", app_state.positions.len()))
                        .size(16.0)
                        .color(Color32::from_rgb(59, 130, 246)),
                );
            });
        });
    });
}

/// Render empty positions state
fn render_empty_positions(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);

        ui.label(RichText::new("📂").size(48.0).color(Color32::GRAY));

        ui.add_space(20.0);

        ui.label(
            RichText::new("No Positions Found")
                .size(18.0)
                .color(Color32::GRAY),
        );

        ui.add_space(10.0);

        ui.label("Execute trades to see your positions here");

        ui.add_space(30.0);

        ui.horizontal(|ui| {
            if success_button("📈 Go to Orders")
                .size(egui::Vec2::new(150.0, 35.0))
                .ui(ui)
                .clicked()
            {
                // This would switch to orders view in the main app
                // For now, just log the action
            }

            ui.add_space(10.0);

            if primary_button("🔄 Refresh")
                .size(egui::Vec2::new(100.0, 35.0))
                .ui(ui)
                .clicked()
            {
                // Refresh positions
            }
        });
    });
}

/// Render the main positions table with real-time updates
fn render_positions_table(ui: &mut Ui, app_state: &mut AppState) {
    let filter = app_state.ui_input.position_filter.to_lowercase();

    ScrollArea::vertical().max_height(600.0).show(ui, |ui| {
        egui::Grid::new("positions_table")
            .num_columns(10)
            .spacing([8.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                // Table header
                ui.label(RichText::new("Symbol").strong());
                ui.label(RichText::new("Exchange").strong());
                ui.label(RichText::new("Product").strong());
                ui.label(RichText::new("Qty").strong());
                ui.label(RichText::new("Avg Price").strong());
                ui.label(RichText::new("LTP").strong());
                ui.label(RichText::new("P&L").strong());
                ui.label(RichText::new("Day P&L").strong());
                ui.label(RichText::new("Change %").strong());
                ui.label(RichText::new("Actions").strong());
                ui.end_row();

                // Table rows
                let positions: Vec<_> = app_state
                    .positions
                    .iter()
                    .map(|entry| entry.value().clone())
                    .collect();
                for position in positions {
                    // Apply filter
                    if !filter.is_empty()
                        && !position.tradingsymbol.to_lowercase().contains(&filter)
                    {
                        continue;
                    }

                    render_position_row(ui, &position, app_state);
                }
            });
    });
}

/// Render individual position row with real-time data
fn render_position_row(ui: &mut Ui, position: &Position, app_state: &mut AppState) {
    // Symbol with color coding based on P&L
    let symbol_color = if position.pnl >= 0.0 {
        Color32::from_rgb(34, 197, 94)
    } else {
        Color32::from_rgb(239, 68, 68)
    };
    ui.colored_label(symbol_color, &position.tradingsymbol);

    // Exchange
    ui.label(&position.exchange);

    // Product
    ui.label(&position.product);

    // Quantity with directional indicator
    let qty_text = if position.quantity > 0 {
        format!("+{}", position.quantity)
    } else {
        position.quantity.to_string()
    };
    let qty_color = if position.quantity > 0 {
        Color32::from_rgb(34, 197, 94)
    } else {
        Color32::from_rgb(239, 68, 68)
    };
    ui.colored_label(qty_color, qty_text);

    // Average price
    ui.label(format!("₹{:.2}", position.average_price));

    // Last traded price (LTP) with real-time updates
    ui.label(
        RichText::new(format!("₹{:.2}", position.last_price))
            .color(Color32::from_rgb(59, 130, 246))
            .strong(),
    );

    // P&L with color coding
    let pnl_color = if position.pnl >= 0.0 {
        Color32::from_rgb(34, 197, 94)
    } else {
        Color32::from_rgb(239, 68, 68)
    };
    let pnl_prefix = if position.pnl >= 0.0 { "+" } else { "" };
    ui.colored_label(pnl_color, format!("{}₹{:.2}", pnl_prefix, position.pnl));

    // Day P&L (unrealized)
    let day_pnl_color = if position.unrealized_pnl >= 0.0 {
        Color32::from_rgb(34, 197, 94)
    } else {
        Color32::from_rgb(239, 68, 68)
    };
    let day_pnl_prefix = if position.unrealized_pnl >= 0.0 {
        "+"
    } else {
        ""
    };
    ui.colored_label(
        day_pnl_color,
        format!("{}₹{:.2}", day_pnl_prefix, position.unrealized_pnl),
    );

    // Change percentage
    let change_pct = if position.average_price > 0.0 {
        ((position.last_price - position.average_price) / position.average_price) * 100.0
    } else {
        0.0
    };
    let change_color = if change_pct >= 0.0 {
        Color32::from_rgb(34, 197, 94)
    } else {
        Color32::from_rgb(239, 68, 68)
    };
    let change_prefix = if change_pct >= 0.0 { "+" } else { "" };
    ui.colored_label(change_color, format!("{}{:.2}%", change_prefix, change_pct));

    // Action buttons
    ui.horizontal(|ui| {
        // Subscribe to ticks button
        if ui.small_button("📡").clicked() {
            app_state.send_command(Command::SubscribeToTicks {
                instrument_tokens: vec![position.instrument_token],
            });

            app_state.add_log(
                LogLevel::Info,
                format!("Subscribed to ticks for {}", position.tradingsymbol),
                Some("positions".to_string()),
            );
        }

        // Quick sell/buy buttons for position management
        if position.quantity > 0 {
            // Show sell button for long positions
            if danger_button("Sell")
                .size(egui::Vec2::new(50.0, 20.0))
                .ui(ui)
                .clicked()
            {
                let order_request = OrderRequest {
                    tradingsymbol: position.tradingsymbol.clone(),
                    exchange: position.exchange.clone(),
                    transaction_type: "SELL".to_string(),
                    order_type: "MARKET".to_string(),
                    quantity: position.quantity.abs(),
                    price: None,
                    product: position.product.clone(),
                    validity: "DAY".to_string(),
                    disclosed_quantity: None,
                    trigger_price: None,
                    squareoff: None,
                    stoploss: None,
                    trailing_stoploss: None,
                    tag: Some("quick_sell".to_string()),
                };

                app_state.send_command(Command::PlaceOrder {
                    details: order_request,
                });
            }
        } else if position.quantity < 0 {
            // Show buy button for short positions
            if success_button("Buy")
                .size(egui::Vec2::new(50.0, 20.0))
                .ui(ui)
                .clicked()
            {
                let order_request = OrderRequest {
                    tradingsymbol: position.tradingsymbol.clone(),
                    exchange: position.exchange.clone(),
                    transaction_type: "BUY".to_string(),
                    order_type: "MARKET".to_string(),
                    quantity: position.quantity.abs(),
                    price: None,
                    product: position.product.clone(),
                    validity: "DAY".to_string(),
                    disclosed_quantity: None,
                    trigger_price: None,
                    squareoff: None,
                    stoploss: None,
                    trailing_stoploss: None,
                    tag: Some("quick_buy".to_string()),
                };

                app_state.send_command(Command::PlaceOrder {
                    details: order_request,
                });
            }
        }
    });

    ui.end_row();
}
