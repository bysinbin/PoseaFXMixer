use eframe::egui::{self, Color32, CornerRadius, Margin, Stroke, Visuals};

pub struct Theme;

impl Theme {
    // FxSound Estetiği Renk Paleti (Obsidian & Coral Crimson)
    pub const BG_DARK: Color32 = Color32::from_rgb(17, 18, 22);       // #111216
    pub const BG_CARD: Color32 = Color32::from_rgb(23, 24, 30);       // #17181E
    pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(30, 32, 40); // #1E2028
    pub const BG_INPUT: Color32 = Color32::from_rgb(14, 15, 18);      // #0E0F12
    pub const BORDER_COLOR: Color32 = Color32::from_rgb(38, 40, 50);  // #262832
    pub const BORDER_LIGHT: Color32 = Color32::from_rgb(52, 56, 70);

    // İkonik Coral Crimson Vurguları
    pub const ACCENT_RED: Color32 = Color32::from_rgb(255, 42, 85);    // #FF2A55
    pub const ACCENT_RED_HOVER: Color32 = Color32::from_rgb(255, 75, 114);
    pub const ACCENT_RED_MUTED: Color32 = Color32::from_rgb(80, 22, 34);
    pub const ACCENT_GREEN: Color32 = Color32::from_rgb(0, 230, 118);

    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(154, 157, 168);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(99, 102, 116);

    pub fn apply(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();

        visuals.override_text_color = Some(Self::TEXT_PRIMARY);
        visuals.panel_fill = Self::BG_DARK;
        visuals.window_fill = Self::BG_CARD;
        visuals.faint_bg_color = Self::BG_CARD;
        visuals.extreme_bg_color = Self::BG_INPUT;

        // FxSound tarzı Trailing Fill (Kızak arkası crimson dolgu)
        visuals.slider_trailing_fill = true;
        visuals.selection.bg_fill = Self::ACCENT_RED;
        visuals.selection.stroke = Stroke::new(1.0, Color32::WHITE);

        // 1. Non-interactive
        visuals.widgets.noninteractive.bg_fill = Self::BG_CARD;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Self::BORDER_COLOR);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);

        // 2. Inactive (Kızak zemin kanalı ve butonlar)
        visuals.widgets.inactive.bg_fill = Self::BG_INPUT;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Self::BORDER_COLOR);
        visuals.widgets.inactive.fg_stroke = Stroke::new(2.0, Self::ACCENT_RED);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(6);

        // 3. Hovered
        visuals.widgets.hovered.bg_fill = Self::ACCENT_RED;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, Color32::WHITE);
        visuals.widgets.hovered.fg_stroke = Stroke::new(2.5, Color32::WHITE);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(6);

        // 4. Active
        visuals.widgets.active.bg_fill = Color32::WHITE;
        visuals.widgets.active.bg_stroke = Stroke::new(2.0, Self::ACCENT_RED);
        visuals.widgets.active.fg_stroke = Stroke::new(3.0, Self::ACCENT_RED);
        visuals.widgets.active.corner_radius = CornerRadius::same(6);

        // 5. Open
        visuals.widgets.open.bg_fill = Self::BG_CARD_HOVER;
        visuals.widgets.open.bg_stroke = Stroke::new(1.2, Self::ACCENT_RED);
        visuals.widgets.open.corner_radius = CornerRadius::same(6);

        ctx.set_visuals(visuals);

        ctx.style_mut_of(egui::Theme::Dark, |style| {
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.button_padding = egui::vec2(12.0, 6.0);
            style.spacing.slider_rail_height = 5.0; // Zarif ve ince FxSound kızağı
            style.spacing.slider_width = 160.0;
        });
    }

    pub fn card_frame() -> egui::Frame {
        egui::Frame::new()
            .fill(Self::BG_CARD)
            .stroke(Stroke::new(1.0, Self::BORDER_COLOR))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(12))
    }
}
