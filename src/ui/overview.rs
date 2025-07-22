use crate::data_structures::*;
use crate::state::{AppState, Command};
use crate::ui::components::{primary_button, success_button};
use chrono::Datelike;
use egui::{Color32, RichText, ScrollArea, Ui};

/// Render comprehensive overview dashboard
/// Optimized for at-a-glance trading information and quick actions
pub fn render_overview(ui: &mut Ui, app_state: &mut AppState) {
    ScrollArea::vertical().show(ui, |ui| {
        // Quick stats row
        render_quick_stats(ui, app_state);

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);

        // Split into two columns
        ui.columns(2, |columns| {
            // Left column: Positions summary
            columns[0].vertical(|ui| {
                render_positions_summary(ui, app_state);
            });

            // Right column: Orders summary and quick actions
            columns[1].vertical(|ui| {
                render_orders_summary(ui, app_state);
                ui.add_space(20.0);
                render_quick_actions(ui, app_state);
            });
        });
    });
}

/// Render quick statistics cards
fn render_quick_stats(ui: &mut Ui, app_state: &AppState) {
    ui.horizontal(|ui| {
        let pnl_data = app_state.calculate_total_pnl();

        // Total PnL card
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Total P&L").strong());
                let color = if pnl_data.total >= 0.0 {
                    Color32::from_rgb(34, 197, 94) // Green
                } else {
                    Color32::from_rgb(239, 68, 68) // Red
                };
                ui.label(
                    RichText::new(format!("₹{:.2}", pnl_data.total))
                        .size(24.0)
                        .color(color),
                );
            });
        });

        ui.add_space(10.0);

        // Day PnL card
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Day P&L").strong());
                let color = if pnl_data.day_pnl >= 0.0 {
                    Color32::from_rgb(34, 197, 94)
                } else {
                    Color32::from_rgb(239, 68, 68)
                };
                ui.label(
                    RichText::new(format!("₹{:.2}", pnl_data.day_pnl))
                        .size(20.0)
                        .color(color),
                );
            });
        });

        ui.add_space(10.0);

        // Positions count
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Positions").strong());
                ui.label(
                    RichText::new(format!("{}", app_state.positions.len()))
                        .size(20.0)
                        .color(Color32::from_rgb(59, 130, 246)),
                );
            });
        });

        ui.add_space(10.0);

        // Orders count
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new("Orders").strong());
                ui.label(
                    RichText::new(format!("{}", app_state.orders.len()))
                        .size(20.0)
                        .color(Color32::from_rgb(59, 130, 246)),
                );
            });
        });
    });
}

/// Render positions summary table
fn render_positions_summary(ui: &mut Ui, app_state: &AppState) {
    ui.label(RichText::new("Recent Positions").size(18.0).strong());
    ui.add_space(10.0);

    if app_state.positions.is_empty() {
        ui.label("No positions found");
        ui.label("Execute trades to see positions here");
        return;
    }

    egui::Grid::new("positions_summary")
        .num_columns(4)
        .spacing([10.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            // Header
            ui.label(RichText::new("Symbol").strong());
            ui.label(RichText::new("Qty").strong());
            ui.label(RichText::new("LTP").strong());
            ui.label(RichText::new("P&L").strong());
            ui.end_row();

            // Show up to 10 positions
            let mut count = 0;
            for entry in app_state.positions.iter() {
                if count >= 10 {
                    break;
                }
                count += 1;

                let position = entry.value();

                ui.label(&position.tradingsymbol);
                ui.label(format!("{}", position.quantity));
                ui.label(format!("₹{:.2}", position.last_price));

                let pnl_color = if position.pnl >= 0.0 {
                    Color32::from_rgb(34, 197, 94)
                } else {
                    Color32::from_rgb(239, 68, 68)
                };
                ui.colored_label(pnl_color, format!("₹{:.2}", position.pnl));
                ui.end_row();
            }

            if app_state.positions.len() > 10 {
                ui.label("");
                ui.label("");
                ui.label("");
                ui.label(format!("... and {} more", app_state.positions.len() - 10));
                ui.end_row();
            }
        });
}

/// Render orders summary table
fn render_orders_summary(ui: &mut Ui, app_state: &AppState) {
    ui.label(RichText::new("Recent Orders").size(18.0).strong());
    ui.add_space(10.0);

    if app_state.orders.is_empty() {
        ui.label("No orders found");
        ui.label("Place orders to see them here");
        return;
    }

    egui::Grid::new("orders_summary")
        .num_columns(4)
        .spacing([10.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            // Header
            ui.label(RichText::new("Symbol").strong());
            ui.label(RichText::new("Type").strong());
            ui.label(RichText::new("Qty").strong());
            ui.label(RichText::new("Status").strong());
            ui.end_row();

            // Show up to 8 orders
            let mut count = 0;
            for entry in app_state.orders.iter() {
                if count >= 8 {
                    break;
                }
                count += 1;

                let order = entry.value();

                ui.label(&order.tradingsymbol);
                ui.label(&order.transaction_type);
                ui.label(format!("{}", order.quantity));

                let status_color = match order.status {
                    OrderStatus::Complete => Color32::from_rgb(34, 197, 94),
                    OrderStatus::Open => Color32::from_rgb(59, 130, 246),
                    OrderStatus::Cancelled => Color32::from_rgb(107, 114, 128),
                    OrderStatus::Rejected => Color32::from_rgb(239, 68, 68),
                    _ => Color32::from_rgb(245, 158, 11),
                };
                ui.colored_label(status_color, format!("{:?}", order.status));
                ui.end_row();
            }

            if app_state.orders.len() > 8 {
                ui.label("");
                ui.label("");
                ui.label("");
                ui.label(format!("... and {} more", app_state.orders.len() - 8));
                ui.end_row();
            }
        });
}

/// Render quick action buttons
fn render_quick_actions(ui: &mut Ui, app_state: &mut AppState) {
    ui.label(RichText::new("Quick Actions").size(18.0).strong());
    ui.add_space(10.0);

    ui.vertical(|ui| {
        // Refresh data button
        if primary_button("🔄 Refresh All Data")
            .size(egui::Vec2::new(200.0, 35.0))
            .ui(ui)
            .clicked()
        {
            app_state.send_command(Command::FetchPositions);
            app_state.send_command(Command::FetchOrders);

            app_state.add_log(
                LogLevel::Info,
                "Refreshing all data...".to_string(),
                Some("overview".to_string()),
            );
        }

        ui.add_space(10.0);

        // Subscribe to popular instruments
        if success_button("📡 Subscribe to NIFTY 50")
            .size(egui::Vec2::new(200.0, 35.0))
            .ui(ui)
            .clicked()
        {
            // Example instrument tokens for NIFTY 50 stocks
            let nifty_tokens = vec![
                256265, // RELIANCE
                424961, // TCS
                779521, // HDFCBANK
                895745, // INFY
                341249, // HINDUNILVR
                        // Add more tokens as needed
            ];

            app_state.send_command(Command::SubscribeToTicks {
                instrument_tokens: nifty_tokens,
            });

            app_state.add_log(
                LogLevel::Info,
                "Subscribed to NIFTY 50 stocks".to_string(),
                Some("overview".to_string()),
            );
        }

        ui.add_space(10.0);

        // Fetch instruments
        if primary_button("📥 Load NSE Instruments")
            .size(egui::Vec2::new(200.0, 35.0))
            .ui(ui)
            .clicked()
        {
            app_state.send_command(Command::FetchInstruments {
                exchange: "NSE".to_string(),
            });

            app_state.add_log(
                LogLevel::Info,
                "Loading NSE instruments...".to_string(),
                Some("overview".to_string()),
            );
        }
    });

    ui.add_space(20.0);

    // Market status indicator
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new("Market Status").strong());
            ui.add_space(5.0);

            // Simple market hours check (9:15 AM to 3:30 PM IST)
            let now = chrono::Local::now();
            let market_open = now.date_naive().and_hms_opt(9, 15, 0).unwrap();
            let market_close = now.date_naive().and_hms_opt(15, 30, 0).unwrap();
            let current_time = now.time();

            let is_weekend =
                now.weekday() == chrono::Weekday::Sat || now.weekday() == chrono::Weekday::Sun;

            if is_weekend {
                ui.colored_label(
                    Color32::from_rgb(107, 114, 128),
                    "🔒 Market Closed (Weekend)",
                );
            } else if current_time >= market_open.time() && current_time <= market_close.time() {
                ui.colored_label(Color32::from_rgb(34, 197, 94), "🟢 Market Open");
            } else {
                ui.colored_label(Color32::from_rgb(239, 68, 68), "🔴 Market Closed");
            }

            ui.label(format!("Current Time: {}", now.format("%H:%M:%S")));
        });
    });
}
