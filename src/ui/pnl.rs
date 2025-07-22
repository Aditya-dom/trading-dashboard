use crate::state::AppState;
use egui::{Color32, RichText, Ui};

/// Render P&L analytics and performance metrics
pub fn render_pnl(ui: &mut Ui, app_state: &AppState) {
    ui.vertical(|ui| {
        ui.label(RichText::new("Profit & Loss").size(24.0).strong());
        ui.add_space(20.0);

        let pnl_data = app_state.calculate_total_pnl();

        // P&L Summary Cards
        ui.horizontal(|ui| {
            // Total P&L
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Total P&L").size(16.0).strong());
                    let color = if pnl_data.total >= 0.0 {
                        Color32::from_rgb(34, 197, 94)
                    } else {
                        Color32::from_rgb(239, 68, 68)
                    };
                    let prefix = if pnl_data.total >= 0.0 { "+" } else { "" };
                    ui.label(
                        RichText::new(format!("{}₹{:.2}", prefix, pnl_data.total))
                            .size(24.0)
                            .color(color),
                    );
                });
            });

            ui.add_space(20.0);

            // Realized P&L
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Realized P&L").size(16.0).strong());
                    let color = if pnl_data.realized >= 0.0 {
                        Color32::from_rgb(34, 197, 94)
                    } else {
                        Color32::from_rgb(239, 68, 68)
                    };
                    let prefix = if pnl_data.realized >= 0.0 { "+" } else { "" };
                    ui.label(
                        RichText::new(format!("{}₹{:.2}", prefix, pnl_data.realized))
                            .size(20.0)
                            .color(color),
                    );
                });
            });

            ui.add_space(20.0);

            // Unrealized P&L
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Unrealized P&L").size(16.0).strong());
                    let color = if pnl_data.unrealized >= 0.0 {
                        Color32::from_rgb(34, 197, 94)
                    } else {
                        Color32::from_rgb(239, 68, 68)
                    };
                    let prefix = if pnl_data.unrealized >= 0.0 { "+" } else { "" };
                    ui.label(
                        RichText::new(format!("{}₹{:.2}", prefix, pnl_data.unrealized))
                            .size(20.0)
                            .color(color),
                    );
                });
            });
        });

        ui.add_space(30.0);
        ui.separator();
        ui.add_space(20.0);

        // Position-wise P&L breakdown
        ui.label(RichText::new("Position-wise P&L").size(18.0).strong());
        ui.add_space(10.0);

        if app_state.positions.is_empty() {
            ui.label("No positions to display P&L");
        } else {
            egui::Grid::new("pnl_table")
                .num_columns(5)
                .spacing([10.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    // Header
                    ui.label(RichText::new("Symbol").strong());
                    ui.label(RichText::new("Quantity").strong());
                    ui.label(RichText::new("Avg Price").strong());
                    ui.label(RichText::new("LTP").strong());
                    ui.label(RichText::new("P&L").strong());
                    ui.end_row();

                    // Rows
                    for entry in app_state.positions.iter() {
                        let position = entry.value();

                        ui.label(&position.tradingsymbol);
                        ui.label(format!("{}", position.quantity));
                        ui.label(format!("₹{:.2}", position.average_price));
                        ui.label(format!("₹{:.2}", position.last_price));

                        let pnl_color = if position.pnl >= 0.0 {
                            Color32::from_rgb(34, 197, 94)
                        } else {
                            Color32::from_rgb(239, 68, 68)
                        };
                        let prefix = if position.pnl >= 0.0 { "+" } else { "" };
                        ui.colored_label(pnl_color, format!("{}₹{:.2}", prefix, position.pnl));

                        ui.end_row();
                    }
                });
        }
    });
}
