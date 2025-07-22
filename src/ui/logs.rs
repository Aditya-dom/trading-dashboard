use crate::data_structures::LogLevel;
use crate::state::AppState;
use egui::{Color32, RichText, ScrollArea, Ui};

/// Render application logs with filtering and color coding
pub fn render_logs(ui: &mut Ui, app_state: &mut AppState) {
    ui.vertical(|ui| {
        // Header
        ui.horizontal(|ui| {
            ui.label(RichText::new("Application Logs").size(24.0).strong());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear Logs").clicked() {
                    let mut logs = app_state.logs.write();
                    logs.clear();
                }
            });
        });

        ui.add_space(10.0);

        // Filter
        ui.horizontal(|ui| {
            ui.label("Filter:");
            let mut filter_copy = app_state.ui_input.log_filter.clone();
            ui.text_edit_singleline(&mut filter_copy);
            app_state.ui_input.log_filter = filter_copy;
        });

        ui.add_space(10.0);

        // Logs display
        let logs = app_state.logs.read();

        ScrollArea::vertical()
            .stick_to_bottom(true)
            .max_height(500.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    for log_entry in logs.iter().rev().take(1000) {
                        // Apply filter
                        if !app_state.ui_input.log_filter.is_empty()
                            && !log_entry
                                .message
                                .to_lowercase()
                                .contains(&app_state.ui_input.log_filter.to_lowercase())
                        {
                            continue;
                        }

                        ui.horizontal(|ui| {
                            // Timestamp
                            ui.label(
                                RichText::new(log_entry.timestamp.format("%H:%M:%S").to_string())
                                    .size(12.0)
                                    .color(Color32::GRAY),
                            );

                            // Level with color coding
                            let (level_text, level_color) = match log_entry.level {
                                LogLevel::Info => ("INFO", Color32::from_rgb(34, 197, 94)),
                                LogLevel::Warning => ("WARN", Color32::from_rgb(245, 158, 11)),
                                LogLevel::Error => ("ERROR", Color32::from_rgb(239, 68, 68)),
                                LogLevel::Debug => ("DEBUG", Color32::from_rgb(107, 114, 128)),
                            };

                            ui.colored_label(level_color, level_text);

                            // Module
                            if let Some(module) = &log_entry.module {
                                ui.label(
                                    RichText::new(format!("[{}]", module))
                                        .size(12.0)
                                        .color(Color32::from_rgb(59, 130, 246)),
                                );
                            }

                            // Message
                            ui.label(&log_entry.message);
                        });
                    }
                });
            });

        ui.add_space(10.0);

        // Log statistics
        ui.horizontal(|ui| {
            ui.label(format!("Total logs: {}", logs.len()));

            // Count by level
            let mut info_count = 0;
            let mut warning_count = 0;
            let mut error_count = 0;
            let mut debug_count = 0;

            for log in logs.iter() {
                match log.level {
                    LogLevel::Info => info_count += 1,
                    LogLevel::Warning => warning_count += 1,
                    LogLevel::Error => error_count += 1,
                    LogLevel::Debug => debug_count += 1,
                }
            }

            ui.separator();
            ui.colored_label(
                Color32::from_rgb(34, 197, 94),
                format!("Info: {}", info_count),
            );
            ui.colored_label(
                Color32::from_rgb(245, 158, 11),
                format!("Warnings: {}", warning_count),
            );
            ui.colored_label(
                Color32::from_rgb(239, 68, 68),
                format!("Errors: {}", error_count),
            );
            ui.colored_label(
                Color32::from_rgb(107, 114, 128),
                format!("Debug: {}", debug_count),
            );
        });
    });
}
