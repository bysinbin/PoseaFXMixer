use eframe::egui::{self, Color32, Pos2, Sense, Stroke, Vec2};
use crate::dsp::NUM_EQ_BANDS;
use crate::gui::theme::Theme;

pub struct EqWidget;

// 31-Bant Standart ISO Frekans Etiketleri
const FREQ_VALUES: [&str; NUM_EQ_BANDS] = [
    "20", "25", "32", "40", "50", "63", "80", "100", "125", "160",
    "200", "250", "315", "400", "500", "630", "800", "1.0", "1.2", "1.6",
    "2.0", "2.5", "3.2", "4.0", "5.0", "6.3", "8.0", "10", "12", "16", "20"
];

const FREQ_UNITS: [&str; NUM_EQ_BANDS] = [
    "Hz", "Hz", "Hz", "Hz", "Hz", "Hz", "Hz", "Hz", "Hz", "Hz",
    "Hz", "Hz", "Hz", "Hz", "Hz", "Hz", "Hz", "kHz", "kHz", "kHz",
    "kHz", "kHz", "kHz", "kHz", "kHz", "kHz", "kHz", "kHz", "kHz", "kHz", "kHz"
];

impl EqWidget {
    pub fn show(
        ui: &mut egui::Ui,
        eq_bands: &mut [f32; NUM_EQ_BANDS],
        height: f32,
        active_band: &mut Option<usize>,
        ignore_input: bool,
    ) -> bool {
        let available_w = ui.available_width();
        let desired_size = Vec2::new(available_w, height.max(160.0));
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        let painter = ui.painter_at(rect);

        // Koyu ve zarif zemin kartı
        painter.rect_filled(rect, 8.0, Theme::BG_CARD);
        painter.rect_stroke(rect, 8.0, Stroke::new(1.0, Color32::from_rgb(32, 34, 44)), egui::StrokeKind::Inside);

        let pad_x = 20.0;
        let pad_top = 20.0;
        let pad_bot = 40.0; // Alt etiketler ve taban arasında geniş, ferah nefes alanı

        let content_w = rect.width() - (pad_x * 2.0);
        let content_h = rect.height() - pad_top - pad_bot;
        let center_y = rect.min.y + pad_top + (content_h * 0.5);

        // 1. 0 dB Referans Kesikli Çizgisi
        let mut dash_x = rect.min.x + pad_x;
        while dash_x < rect.max.x - pad_x {
            painter.line_segment(
                [Pos2::new(dash_x, center_y), Pos2::new((dash_x + 3.0).min(rect.max.x - pad_x), center_y)],
                Stroke::new(1.0, Color32::from_rgb(42, 45, 58)),
            );
            dash_x += 6.0;
        }

        let band_spacing = content_w / (NUM_EQ_BANDS - 1) as f32;

        let mut node_positions = [Pos2::ZERO; NUM_EQ_BANDS];
        let pointer_pos = response.interact_pointer_pos();

        // 2. Dikey Kılavuz Çizgileri ve Frekans Etiketleri
        for i in 0..NUM_EQ_BANDS {
            let x = rect.min.x + pad_x + (i as f32 * band_spacing);
            let gain = eq_bands[i].clamp(-12.0, 12.0);
            let y = center_y - (gain / 12.0) * (content_h * 0.40);
            node_positions[i] = Pos2::new(x, y);

            let is_hovered = pointer_pos.map_or(false, |p| (p.x - x).abs() < (band_spacing * 0.5));
            let guide_color = if is_hovered {
                Color32::from_rgb(60, 64, 80)
            } else {
                Color32::from_rgb(32, 34, 44)
            };

            // Dikey noktalı kılavuz çizgisi
            let mut dot_y = rect.min.y + pad_top;
            while dot_y < rect.max.y - pad_bot + 4.0 {
                painter.circle_filled(Pos2::new(x, dot_y), 0.75, guide_color);
                dot_y += 5.0;
            }

            // Alt frekans etiketleri (Alt kenardan yukarıda, ferah ve okunabilir)
            let text_color = if is_hovered { Color32::WHITE } else { Color32::from_rgb(125, 128, 144) };
            painter.text(
                Pos2::new(x, rect.max.y - 25.0),
                egui::Align2::CENTER_CENTER,
                FREQ_VALUES[i],
                egui::FontId::proportional(8.5),
                text_color,
            );
            painter.text(
                Pos2::new(x, rect.max.y - 13.0),
                egui::Align2::CENTER_CENTER,
                FREQ_UNITS[i],
                egui::FontId::proportional(7.5),
                Color32::from_rgb(75, 78, 92),
            );
        }

        // 3. Akıcı Catmull-Rom Spline Eğrisi Hesabı
        let mut curve_points = Vec::with_capacity(NUM_EQ_BANDS * 8);
        let steps_per_segment = 8;

        for i in 0..(NUM_EQ_BANDS - 1) {
            let p0 = if i == 0 { node_positions[0] } else { node_positions[i - 1] };
            let p1 = node_positions[i];
            let p2 = node_positions[i + 1];
            let p3 = if i + 2 < NUM_EQ_BANDS { node_positions[i + 2] } else { p2 };

            for step in 0..=steps_per_segment {
                let t = step as f32 / steps_per_segment as f32;
                let t2 = t * t;
                let t3 = t2 * t;

                let x = 0.5 * ((2.0 * p1.x) +
                    (-p0.x + p2.x) * t +
                    (2.0 * p0.x - 5.0 * p1.x + 4.0 * p2.x - p3.x) * t2 +
                    (-p0.x + 3.0 * p1.x - 3.0 * p2.x + p3.x) * t3);

                let y = 0.5 * ((2.0 * p1.y) +
                    (-p0.y + p2.y) * t +
                    (2.0 * p0.y - 5.0 * p1.y + 4.0 * p2.y - p3.y) * t2 +
                    (-p0.y + 3.0 * p1.y - 3.0 * p2.y + p3.y) * t3);

                curve_points.push(Pos2::new(x, y));
            }
        }

        // 4. Eğrinin altındaki yumuşak kırmızı gradyan dolgu
        if !curve_points.is_empty() {
            let mut fill_mesh = egui::Mesh::default();
            let fill_color = Color32::from_rgba_unmultiplied(255, 42, 85, 26);
            let zero_color = Color32::from_rgba_unmultiplied(255, 42, 85, 3);

            for window in curve_points.windows(2) {
                let pt1 = window[0];
                let pt2 = window[1];

                let idx = fill_mesh.vertices.len() as u32;
                fill_mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(pt1.x, center_y), uv: egui::epaint::WHITE_UV, color: zero_color });
                fill_mesh.vertices.push(egui::epaint::Vertex { pos: pt1, uv: egui::epaint::WHITE_UV, color: fill_color });
                fill_mesh.vertices.push(egui::epaint::Vertex { pos: pt2, uv: egui::epaint::WHITE_UV, color: fill_color });
                fill_mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(pt2.x, center_y), uv: egui::epaint::WHITE_UV, color: zero_color });

                fill_mesh.add_triangle(idx, idx + 1, idx + 2);
                fill_mesh.add_triangle(idx, idx + 2, idx + 3);
            }
            painter.add(fill_mesh);
        }

        // 5. Parlak Coral Crimson Ana Eğri Çizgisi
        for window in curve_points.windows(2) {
            painter.line_segment([window[0], window[1]], Stroke::new(2.4, Theme::ACCENT_RED));
        }

        // 6. Düğüm Noktaları (Halka Şeklinde) ve Kazanç Rozetleri
        for i in 0..NUM_EQ_BANDS {
            let pos = node_positions[i];
            let is_closest = pointer_pos.map_or(false, |p| (p - pos).length() < 12.0);

            let r = if is_closest { 4.6 } else { 3.6 };

            // Halka: Coral Crimson dış çember + Koyu iç dolgu
            painter.circle_filled(pos, r, Color32::from_rgb(18, 19, 25));
            painter.circle_stroke(pos, r, Stroke::new(1.8, Theme::ACCENT_RED));

            // Kazanç Değeri (+4, 0, -6)
            let val = eq_bands[i];
            let sign = if val > 0.0 { "+" } else { "" };
            let val_text = format!("{}{:.0}", sign, val);

            let label_y = if val >= 0.0 { pos.y - 9.0 } else { pos.y + 9.0 };
            let label_color = if is_closest {
                Theme::ACCENT_RED
            } else if val.abs() > 0.01 {
                Color32::WHITE
            } else {
                Color32::from_rgb(120, 124, 140)
            };

            painter.text(
                Pos2::new(pos.x, label_y),
                egui::Align2::CENTER_CENTER,
                val_text,
                egui::FontId::proportional(8.0),
                label_color,
            );
        }

        // 7. Etkileşim: Sürükleme ve Çift Tıklama (Kilitli Güvenli Etkileşim)
        let mut changed = false;

        let pointer_down = ui.input(|i| i.pointer.primary_down());
        if !pointer_down {
            *active_band = None;
        }

        if !ignore_input {
            if response.double_clicked() {
                if let Some(mouse_pos) = pointer_pos {
                    for (i, p) in node_positions.iter().enumerate() {
                        let d = (p.x - mouse_pos.x).abs();
                        if d <= (band_spacing * 0.65).max(10.0) {
                            eq_bands[i] = 0.0;
                            changed = true;
                            break;
                        }
                    }
                }
            } else {
                // Yeni bir tıklama başladığında ve henüz aktif bir bant seçilmemişken
                if pointer_down && active_band.is_none() && ui.input(|i| i.pointer.any_pressed()) && response.hovered() {
                    if let Some(mouse_pos) = pointer_pos {
                        let in_y_range = mouse_pos.y >= (rect.min.y + pad_top - 12.0)
                            && mouse_pos.y <= (rect.max.y - pad_bot + 12.0);

                        if in_y_range {
                            let mut best_idx = None;
                            let mut min_dist_x = f32::MAX;

                            for (i, node_pos) in node_positions.iter().enumerate() {
                                let dx = (mouse_pos.x - node_pos.x).abs();
                                if dx < min_dist_x {
                                    min_dist_x = dx;
                                    if dx <= (band_spacing * 0.60).max(10.0) {
                                        best_idx = Some(i);
                                    }
                                }
                            }

                            *active_band = best_idx;
                        }
                    }
                }

                // Kilitlenmiş aktif bant varsa, sadece o bandın kazancını güncelle
                if pointer_down {
                    if let Some(idx) = *active_band {
                        if let Some(mouse_pos) = pointer_pos {
                            let clamped_y = mouse_pos.y.clamp(rect.min.y + pad_top, rect.max.y - pad_bot);
                            let norm = (center_y - clamped_y) / (content_h * 0.40);
                            let new_gain = (norm * 12.0).round().clamp(-12.0, 12.0);
                            if (eq_bands[idx] - new_gain).abs() > 0.05 {
                                eq_bands[idx] = new_gain;
                                changed = true;
                            }
                        }
                    }
                }
            }
        } else {
            *active_band = None;
        }

        changed
    }
}
