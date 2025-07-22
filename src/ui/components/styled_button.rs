use egui::{Button, Color32, Response, Ui, Vec2};

/// High-performance styled button component for trading actions
/// Optimized for minimal allocations and fast rendering
pub struct StyledButton {
    text: String,
    style: ButtonStyle,
    size: Option<Vec2>,
    enabled: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
    Buy,
    Sell,
}

impl StyledButton {
    /// Create new styled button
    pub fn new(text: impl Into<String>, style: ButtonStyle) -> Self {
        Self {
            text: text.into(),
            style,
            size: None,
            enabled: true,
        }
    }

    /// Set button size
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }

    /// Set button enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Render the button with optimized styling
    pub fn ui(self, ui: &mut Ui) -> Response {
        let (bg_color, text_color, hover_color) = self.get_colors();

        // Apply styling
        let style = ui.style_mut();
        style.visuals.widgets.inactive.bg_fill = bg_color;
        style.visuals.widgets.hovered.bg_fill = hover_color;
        style.visuals.widgets.active.bg_fill = hover_color;
        style.visuals.widgets.inactive.fg_stroke.color = text_color;
        style.visuals.widgets.hovered.fg_stroke.color = text_color;
        style.visuals.widgets.active.fg_stroke.color = text_color;

        let mut button = Button::new(&self.text);

        if let Some(size) = self.size {
            button = button.min_size(size);
        }

        ui.add_enabled(self.enabled, button)
    }

    /// Get color scheme for button style
    fn get_colors(&self) -> (Color32, Color32, Color32) {
        match self.style {
            ButtonStyle::Primary => (
                Color32::from_rgb(59, 130, 246), // Blue
                Color32::WHITE,
                Color32::from_rgb(37, 99, 235), // Darker blue
            ),
            ButtonStyle::Secondary => (
                Color32::from_rgb(107, 114, 128), // Gray
                Color32::WHITE,
                Color32::from_rgb(75, 85, 99), // Darker gray
            ),
            ButtonStyle::Success => (
                Color32::from_rgb(34, 197, 94), // Green
                Color32::WHITE,
                Color32::from_rgb(22, 163, 74), // Darker green
            ),
            ButtonStyle::Warning => (
                Color32::from_rgb(245, 158, 11), // Amber
                Color32::WHITE,
                Color32::from_rgb(217, 119, 6), // Darker amber
            ),
            ButtonStyle::Danger => (
                Color32::from_rgb(239, 68, 68), // Red
                Color32::WHITE,
                Color32::from_rgb(220, 38, 38), // Darker red
            ),
            ButtonStyle::Buy => (
                Color32::from_rgb(16, 185, 129), // Emerald (buy green)
                Color32::WHITE,
                Color32::from_rgb(5, 150, 105), // Darker emerald
            ),
            ButtonStyle::Sell => (
                Color32::from_rgb(239, 68, 68), // Red (sell red)
                Color32::WHITE,
                Color32::from_rgb(220, 38, 38), // Darker red
            ),
        }
    }
}

/// Quick helper functions for common button types
pub fn primary_button(text: impl Into<String>) -> StyledButton {
    StyledButton::new(text, ButtonStyle::Primary)
}

pub fn secondary_button(text: impl Into<String>) -> StyledButton {
    StyledButton::new(text, ButtonStyle::Secondary)
}

pub fn success_button(text: impl Into<String>) -> StyledButton {
    StyledButton::new(text, ButtonStyle::Success)
}

pub fn warning_button(text: impl Into<String>) -> StyledButton {
    StyledButton::new(text, ButtonStyle::Warning)
}

pub fn danger_button(text: impl Into<String>) -> StyledButton {
    StyledButton::new(text, ButtonStyle::Danger)
}

pub fn buy_button(text: impl Into<String>) -> StyledButton {
    StyledButton::new(text, ButtonStyle::Buy)
}

pub fn sell_button(text: impl Into<String>) -> StyledButton {
    StyledButton::new(text, ButtonStyle::Sell)
}
