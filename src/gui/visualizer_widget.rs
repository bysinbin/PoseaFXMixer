use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use crate::dsp::visualizer_data::{VisualizerData, NUM_SPECTRUM_BARS};
use crate::gui::theme::Theme;

pub struct VisualizerWidget;

impl VisualizerWidget {
    pub fn show(ui: &mut egui::Ui, visualizer: &VisualizerData, is_dsp_enabled: bool, height: f32) {
        let available_w = ui.available_width();
        let desired_size = Vec2::new(available_w, height.clamp(70.0, 140.0));
        let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

        let painter = ui.painter_at(rect);

        // Koyu ve zarif zemin kartı
        painter.rect_filled(rect, 8.0, Theme::BG_CARD);
        painter.rect_stroke(rect, 8.0, Stroke::new(1.0, Color32::from_rgb(32, 34, 44)), egui::StrokeKind::Inside);

        let bands = visualizer.get_bands();
        let center_y = rect.center().y;
        let max_half_h = (rect.height() * 0.5) - 8.0;

        let num_bars = NUM_SPECTRUM_BARS;
        // Barları kutunun genişliğine orantılı şekilde yay
        let total_usable_w = rect.width() - 32.0;
        let bar_pitch = total_usable_w / num_bars as f32;
        let bar_width = (bar_pitch * 0.55).clamp(2.5, 4.0);
        let start_x = rect.min.x + 16.0;

        for i in 0..num_bars {
            let x = start_x + (i as f32 * bar_pitch);
            let raw_level = if is_dsp_enabled { bands[i] } else { bands[i] * 0.20 };

            if raw_level > 0.04 {
                // Symmetrical bipolar soundwave
                let half_h = (raw_level * max_half_h).clamp(3.0, max_half_h);
                let bar_rect = Rect::from_min_max(
                    Pos2::new(x, center_y - half_h),
                    Pos2::new(x + bar_width, center_y + half_h),
                );

                let color = if is_dsp_enabled {
                    Theme::ACCENT_RED
                } else {
                    Color32::from_rgb(90, 94, 110)
                };

                painter.rect_filled(bar_rect, 2.0, color);
            } else {
                // Sessiz dinlenme noktası
                let dot_center = Pos2::new(x + (bar_width * 0.5), center_y);
                painter.circle_filled(dot_center, 1.2, Color32::from_rgb(42, 45, 58));
            }
        }

        ui.ctx().request_repaint();
    }
}
