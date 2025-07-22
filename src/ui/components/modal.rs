use egui::{Align2, Area, Color32, Frame, Id, Order, Pos2, Response, Sense, Ui, Vec2, Window};

/// High-performance modal component for trading dialogs
/// Optimized for responsive interactions and minimal latency
pub struct Modal {
    id: Id,
    title: String,
    size: Option<Vec2>,
    resizable: bool,
    closable: bool,
    backdrop_color: Color32,
}

impl Modal {
    /// Create new modal dialog
    pub fn new(id: impl Into<Id>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            size: None,
            resizable: false,
            closable: true,
            backdrop_color: Color32::from_black_alpha(128),
        }
    }

    /// Set modal size
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }

    /// Set whether modal is resizable
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set whether modal can be closed
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Set backdrop color
    pub fn backdrop_color(mut self, color: Color32) -> Self {
        self.backdrop_color = color;
        self
    }

    /// Show modal with content
    pub fn show<R>(
        self,
        ctx: &egui::Context,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<R> {
        let screen_rect = ctx.screen_rect();

        // Draw backdrop
        Area::new("modal_backdrop".into())
            .order(Order::Background)
            .show(ctx, |ui| {
                let backdrop_rect = screen_rect;
                ui.allocate_response(backdrop_rect.size(), Sense::click());

                let painter = ui.painter();
                painter.rect_filled(backdrop_rect, 0.0, self.backdrop_color);
            });

        // Show modal window
        let mut window = Window::new(&self.title)
            .id(self.id)
            .order(Order::Foreground)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .collapsible(false)
            .resizable(self.resizable);

        if let Some(size) = self.size {
            window = window.fixed_size(size);
        }

        if !self.closable {
            window = window.title_bar(false);
        }

        let mut result = None;

        window.show(ctx, |ui| {
            result = Some(add_contents(ui));
        });

        result
    }

    /// Show confirmation modal
    pub fn confirmation<R>(
        ctx: &egui::Context,
        id: impl Into<Id>,
        title: impl Into<String>,
        message: impl Into<String>,
        on_confirm: impl FnOnce() -> R,
        on_cancel: impl FnOnce() -> R,
    ) -> Option<R> {
        let modal = Modal::new(id, title).size(Vec2::new(400.0, 150.0));

        modal
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(message.into());
                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        ui.add_space(50.0);

                        if ui.button("Confirm").clicked() {
                            return Some(on_confirm());
                        }

                        ui.add_space(20.0);

                        if ui.button("Cancel").clicked() {
                            return Some(on_cancel());
                        }

                        None
                    })
                    .inner
                })
                .inner
            })
            .flatten()
    }

    /// Show error modal
    pub fn error(
        ctx: &egui::Context,
        id: impl Into<Id>,
        title: impl Into<String>,
        error_message: impl Into<String>,
    ) -> bool {
        let modal = Modal::new(id, title).size(Vec2::new(450.0, 200.0));

        modal
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    // Error icon (red circle with X)
                    let painter = ui.painter();
                    let icon_center = ui.cursor().center() + Vec2::new(0.0, 15.0);
                    painter.circle_filled(icon_center, 20.0, Color32::from_rgb(239, 68, 68));
                    painter.text(
                        icon_center,
                        Align2::CENTER_CENTER,
                        "✗",
                        egui::FontId::proportional(24.0),
                        Color32::WHITE,
                    );

                    ui.add_space(40.0);
                    ui.label(error_message.into());
                    ui.add_space(20.0);

                    ui.button("OK").clicked()
                })
                .inner
            })
            .unwrap_or(false)
    }

    /// Show loading modal
    pub fn loading(ctx: &egui::Context, id: impl Into<Id>, message: impl Into<String>) {
        let modal = Modal::new(id, "Loading...")
            .size(Vec2::new(300.0, 120.0))
            .closable(false);

        modal.show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.spinner();
                ui.add_space(10.0);
                ui.label(message.into());
            });
        });
    }
}
