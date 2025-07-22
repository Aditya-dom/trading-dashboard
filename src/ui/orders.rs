use crate::data_structures::*;
use crate::state::{AppState, Command};
use crate::ui::components::{
    buy_button, danger_button, primary_button, sell_button, success_button,
};
use egui::{Color32, RichText, ScrollArea, Ui};

/// Render orders management interface with filtering and actions
pub fn render_orders(ui: &mut Ui, app_state: &mut AppState) {
    ui.vertical(|ui| {
        // Header
        ui.horizontal(|ui| {
            ui.label(RichText::new("Orders").size(24.0).strong());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if success_button("+ New Order")
                    .size(egui::Vec2::new(120.0, 30.0))
                    .ui(ui)
                    .clicked()
                {
                    app_state.ui_input.show_order_dialog = true;
                }

                ui.add_space(10.0);

                if primary_button("🔄 Refresh")
                    .size(egui::Vec2::new(100.0, 30.0))
                    .ui(ui)
                    .clicked()
                {
                    app_state.send_command(Command::FetchOrders);
                }
            });
        });

        ui.add_space(10.0);

        // Filter
        ui.horizontal(|ui| {
            ui.label("Filter by symbol:");
            ui.text_edit_singleline(&mut app_state.ui_input.order_filter);

            if ui.button("Clear").clicked() {
                app_state.ui_input.order_filter.clear();
            }
        });

        ui.add_space(10.0);

        // Orders table
        if app_state.orders.is_empty() {
            render_empty_orders(ui);
        } else {
            render_orders_table(ui, app_state);
        }

        // New order dialog
        if app_state.ui_input.show_order_dialog {
            render_order_dialog(ui, app_state);
        }
    });
}

fn render_empty_orders(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        ui.label(RichText::new("📋").size(48.0).color(Color32::GRAY));
        ui.add_space(20.0);
        ui.label(
            RichText::new("No Orders Found")
                .size(18.0)
                .color(Color32::GRAY),
        );
        ui.label("Place orders to see them here");
    });
}

fn render_orders_table(ui: &mut Ui, app_state: &mut AppState) {
    let filtered_orders = app_state.get_filtered_orders(&app_state.ui_input.order_filter);

    ScrollArea::vertical().max_height(600.0).show(ui, |ui| {
        egui::Grid::new("orders_table")
            .num_columns(8)
            .spacing([8.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                // Header
                ui.label(RichText::new("Symbol").strong());
                ui.label(RichText::new("Type").strong());
                ui.label(RichText::new("Qty").strong());
                ui.label(RichText::new("Price").strong());
                ui.label(RichText::new("Status").strong());
                ui.label(RichText::new("Filled").strong());
                ui.label(RichText::new("Time").strong());
                ui.label(RichText::new("Actions").strong());
                ui.end_row();

                // Rows
                for order in &filtered_orders {
                    ui.label(&order.tradingsymbol);

                    let type_color = if order.transaction_type == "BUY" {
                        Color32::from_rgb(34, 197, 94)
                    } else {
                        Color32::from_rgb(239, 68, 68)
                    };
                    ui.colored_label(type_color, &order.transaction_type);

                    ui.label(format!("{}", order.quantity));
                    ui.label(format!("₹{:.2}", order.price));

                    let status_color = match order.status {
                        OrderStatus::Complete => Color32::from_rgb(34, 197, 94),
                        OrderStatus::Open => Color32::from_rgb(59, 130, 246),
                        OrderStatus::Cancelled => Color32::from_rgb(107, 114, 128),
                        OrderStatus::Rejected => Color32::from_rgb(239, 68, 68),
                        _ => Color32::from_rgb(245, 158, 11),
                    };
                    ui.colored_label(status_color, format!("{:?}", order.status));

                    ui.label(format!("{}/{}", order.filled_quantity, order.quantity));
                    ui.label(order.order_timestamp.format("%H:%M:%S").to_string());

                    // Actions
                    ui.horizontal(|ui| {
                        if matches!(order.status, OrderStatus::Open | OrderStatus::Trigger) {
                            if danger_button("Cancel")
                                .size(egui::Vec2::new(60.0, 20.0))
                                .ui(ui)
                                .clicked()
                            {
                                app_state.send_command(Command::CancelOrder {
                                    order_id: order.order_id.clone(),
                                });
                            }
                        }
                    });

                    ui.end_row();
                }
            });
    });
}

fn render_order_dialog(ui: &mut Ui, app_state: &mut AppState) {
    // Simple order dialog - in a real app, this would be a modal
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new("Place New Order").size(18.0).strong());
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Symbol:");
                ui.text_edit_singleline(&mut app_state.ui_input.order_symbol_input);
            });

            ui.horizontal(|ui| {
                ui.label("Quantity:");
                ui.text_edit_singleline(&mut app_state.ui_input.order_quantity_input);
            });

            ui.horizontal(|ui| {
                ui.label("Price:");
                ui.text_edit_singleline(&mut app_state.ui_input.order_price_input);
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if buy_button("Buy")
                    .size(egui::Vec2::new(80.0, 30.0))
                    .ui(ui)
                    .clicked()
                {
                    place_order(app_state, "BUY");
                }

                ui.add_space(10.0);

                if sell_button("Sell")
                    .size(egui::Vec2::new(80.0, 30.0))
                    .ui(ui)
                    .clicked()
                {
                    place_order(app_state, "SELL");
                }

                ui.add_space(20.0);

                if ui.button("Cancel").clicked() {
                    app_state.ui_input.show_order_dialog = false;
                }
            });
        });
    });
}

fn place_order(app_state: &mut AppState, transaction_type: &str) {
    let quantity: i32 = app_state.ui_input.order_quantity_input.parse().unwrap_or(0);
    let price: f64 = app_state.ui_input.order_price_input.parse().unwrap_or(0.0);

    if !app_state.ui_input.order_symbol_input.is_empty() && quantity > 0 {
        let order_request = OrderRequest {
            tradingsymbol: app_state.ui_input.order_symbol_input.clone(),
            exchange: "NSE".to_string(),
            transaction_type: transaction_type.to_string(),
            order_type: if price > 0.0 { "LIMIT" } else { "MARKET" }.to_string(),
            quantity,
            price: if price > 0.0 { Some(price) } else { None },
            product: "MIS".to_string(),
            validity: "DAY".to_string(),
            disclosed_quantity: None,
            trigger_price: None,
            squareoff: None,
            stoploss: None,
            trailing_stoploss: None,
            tag: Some("manual_order".to_string()),
        };

        app_state.send_command(Command::PlaceOrder {
            details: order_request,
        });

        // Clear inputs
        app_state.ui_input.order_symbol_input.clear();
        app_state.ui_input.order_quantity_input.clear();
        app_state.ui_input.order_price_input.clear();
        app_state.ui_input.show_order_dialog = false;
    }
}
