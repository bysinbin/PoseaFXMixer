use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use cpal::traits::{DeviceTrait, HostTrait};
use eframe::egui::{self, Color32, CornerRadius, CursorIcon, Margin, Pos2, Rect, ResizeDirection, RichText, Sense, Stroke, Vec2};
use parking_lot::RwLock;

use crate::dsp::visualizer_data::VisualizerData;
use crate::dsp::DspSettings;
use crate::gui::eq_widget::EqWidget;
use crate::gui::theme::Theme;
use crate::gui::tray::{TrayAction, TrayManager};
use crate::gui::visualizer_widget::VisualizerWidget;
use crate::utils::autostart;

#[derive(PartialEq, Clone, Copy)]
enum PreferencesTab {
    Audio,
    General,
    Help,
}

pub struct PoseaMixerApp {
    settings: Arc<RwLock<DspSettings>>,
    visualizer: Arc<VisualizerData>,
    selected_input: Arc<RwLock<Option<String>>>,
    selected_output: Arc<RwLock<Option<String>>>,
    reload_requested: Arc<AtomicBool>,
    tray_manager: Option<TrayManager>,

    // UI Durumları
    minimize_to_tray: bool,
    is_window_visible: bool,
    autostart_enabled: bool,
    show_preferences_modal: bool,
    pref_active_tab: PreferencesTab,
    selected_preset_name: String,
    cached_devices: Vec<String>,

    // Özel Açılır Menü (Custom Dropdown) Durumları
    preset_dropdown_open: bool,
    device_dropdown_open: bool,

    // Tercihler İçin Ek Ses Ayarları
    master_volume_db: f32, // -12 .. +12 dB
    normalization_db: f32, // 0 .. 12 dB
    filter_q: f32,         // 0.5 .. 3.0x
    balance: f32,          // -1.0 .. +1.0

    // Pencere Boyutlandırma & Kilitli EQ Durumları
    is_resizing_window: bool,
    active_eq_band: Option<usize>,
}

impl PoseaMixerApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        settings: Arc<RwLock<DspSettings>>,
        visualizer: Arc<VisualizerData>,
        selected_input: Arc<RwLock<Option<String>>>,
        selected_output: Arc<RwLock<Option<String>>>,
        reload_requested: Arc<AtomicBool>,
    ) -> Self {
        Theme::apply(&_cc.egui_ctx);

        let tray_manager = TrayManager::new();
        let autostart_enabled = autostart::is_autostart_enabled();
        let cached_devices = Self::scan_devices();

        Self {
            settings,
            visualizer,
            selected_input,
            selected_output,
            reload_requested,
            tray_manager,
            minimize_to_tray: true,
            is_window_visible: true,
            autostart_enabled,
            show_preferences_modal: false,
            pref_active_tab: PreferencesTab::Audio,
            selected_preset_name: "Music".to_string(),
            cached_devices,
            preset_dropdown_open: false,
            device_dropdown_open: false,
            master_volume_db: 0.0,
            normalization_db: 0.0,
            filter_q: 1.0,
            balance: 0.0,
            is_resizing_window: false,
            active_eq_band: None,
        }
    }

    fn scan_devices() -> Vec<String> {
        let host = cpal::default_host();
        let mut names = Vec::new();
        if let Ok(devs) = host.output_devices() {
            for d in devs {
                if let Ok(desc) = d.description() {
                    let name = desc.name().to_string();
                    if !name.contains("Posea Audio Virtual") {
                        names.push(name);
                    }
                }
            }
        }
        names
    }

    /// FxSound İkonik Kızak Çizici (Halka Düğme, Kırmızı İlerleme Hattı ve Çakışmayan Sağ Değer)
    fn render_fx_slider(ui: &mut egui::Ui, label: &str, val: &mut f32, ignore_input: bool) -> bool {
        let mut changed = false;
        let slider_w = ui.available_width().max(180.0);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(slider_w, 32.0), Sense::click_and_drag());

        let painter = ui.painter_at(rect);

        // 1. Üst Etiket
        painter.text(
            Pos2::new(rect.min.x, rect.min.y + 6.0),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(215, 218, 228),
        );

        let track_start_x = rect.min.x;
        let track_end_x = rect.max.x - 38.0; // 38px net boşluk (değer asla başparmak ile çakışmaz)
        let track_w = (track_end_x - track_start_x).max(10.0);
        let track_y = rect.min.y + 22.0;

        // Etkileşim: Sürükleme veya Tıklama (Sadece pencere boyutlandırılmıyorsa)
        if !ignore_input && (response.dragged() || response.clicked()) {
            if let Some(pos) = response.interact_pointer_pos() {
                let ratio = ((pos.x - track_start_x) / track_w).clamp(0.0, 1.0);
                let new_val = (ratio * 10.0).round();
                if (*val - new_val).abs() > 0.01 {
                    *val = new_val;
                    changed = true;
                }
            }
        }

        let current_ratio = (*val / 10.0).clamp(0.0, 1.0);
        let thumb_x = track_start_x + (current_ratio * track_w);

        // 2. Pasif Kanal (Gri)
        painter.line_segment(
            [Pos2::new(track_start_x, track_y), Pos2::new(track_end_x, track_y)],
            Stroke::new(2.5, Color32::from_rgb(42, 44, 56)),
        );

        // 3. Aktif Kanal (Coral Crimson Kırmızı)
        if current_ratio > 0.005 {
            painter.line_segment(
                [Pos2::new(track_start_x, track_y), Pos2::new(thumb_x, track_y)],
                Stroke::new(2.5, Theme::ACCENT_RED),
            );
        }

        // 4. Halka Düğme (Thumb Ring: Dışı Kırmızı, İçi Koyu)
        let is_hovered = response.hovered();
        let r = if is_hovered { 6.0 } else { 5.0 };
        painter.circle_filled(Pos2::new(thumb_x, track_y), r, Color32::from_rgb(17, 18, 22));
        painter.circle_stroke(Pos2::new(thumb_x, track_y), r, Stroke::new(2.0, Theme::ACCENT_RED));

        // 5. Değer Rozeti (Sağ Uçta Beyaz Rakam, asla üst üste binmez)
        let val_color = if *val > 0.0 { Color32::WHITE } else { Color32::from_rgb(130, 134, 150) };
        painter.text(
            Pos2::new(rect.max.x - 6.0, track_y),
            egui::Align2::RIGHT_CENTER,
            format!("{:.0}", *val),
            egui::FontId::proportional(13.0),
            val_color,
        );

        changed
    }

    /// Tercihler Modalı İçin Özel Yatay Satır Kaydırıcısı (FxSound media_1793383686130_286d1f53.png Birebir)
    fn render_pref_slider(ui: &mut egui::Ui, label: &str, val: &mut f32, min: f32, max: f32, display_val: &str) -> bool {
        let mut changed = false;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(390.0, 26.0), Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        // Sol: Etiket
        painter.text(
            Pos2::new(rect.min.x, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(12.0),
            Color32::from_rgb(170, 174, 188),
        );

        let track_start_x = rect.min.x + 130.0;
        let track_end_x = rect.max.x - 48.0;
        let track_w = (track_end_x - track_start_x).max(10.0);
        let track_y = rect.center().y;

        if response.dragged() || response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let ratio = ((pos.x - track_start_x) / track_w).clamp(0.0, 1.0);
                let new_val = min + ratio * (max - min);
                if (*val - new_val).abs() > 0.02 {
                    *val = new_val;
                    changed = true;
                }
            }
        }

        let ratio = ((*val - min) / (max - min)).clamp(0.0, 1.0);
        let thumb_x = track_start_x + (ratio * track_w);

        // Pasif kanal
        painter.line_segment(
            [Pos2::new(track_start_x, track_y), Pos2::new(track_end_x, track_y)],
            Stroke::new(2.2, Color32::from_rgb(42, 45, 58)),
        );

        // Aktif kanal
        if ratio > 0.005 {
            painter.line_segment(
                [Pos2::new(track_start_x, track_y), Pos2::new(thumb_x, track_y)],
                Stroke::new(2.2, Theme::ACCENT_RED),
            );
        }

        // Halka düğme
        let r = if response.hovered() { 5.6 } else { 4.6 };
        painter.circle_filled(Pos2::new(thumb_x, track_y), r, Color32::from_rgb(18, 19, 25));
        painter.circle_stroke(Pos2::new(thumb_x, track_y), r, Stroke::new(1.8, Theme::ACCENT_RED));

        // Sağdaki değer yazısı
        painter.text(
            Pos2::new(rect.max.x - 4.0, track_y),
            egui::Align2::RIGHT_CENTER,
            display_val,
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );

        changed
    }

    /// Çerçevesiz Pencere İçin Kenar Boyutlandırma (8 Yönlü Resize Tutamaçları)
    fn handle_window_resizing(&mut self, ctx: &egui::Context, win_rect: Rect) -> bool {
        let border_pad = 10.0;
        let mut is_on_border = false;

        let pointer_down = ctx.input(|i| i.pointer.primary_down());
        if !pointer_down {
            self.is_resizing_window = false;
        }

        // Eğer kullanıcı zaten bir EQ bandını aktif olarak sürüklüyorsa, boyutlandırma tetikleme
        if self.active_eq_band.is_some() {
            return false;
        }

        if let Some(pos) = ctx.pointer_latest_pos() {
            let on_left = pos.x >= win_rect.min.x && pos.x <= win_rect.min.x + border_pad;
            let on_right = pos.x >= win_rect.max.x - border_pad && pos.x <= win_rect.max.x;
            let on_top = pos.y >= win_rect.min.y && pos.y <= win_rect.min.y + border_pad;
            let on_bottom = pos.y >= win_rect.max.y - border_pad && pos.y <= win_rect.max.y;

            is_on_border = on_left || on_right || on_top || on_bottom;

            let (dir, cursor) = match (on_left, on_right, on_top, on_bottom) {
                (true, _, true, _) => (Some(ResizeDirection::NorthWest), CursorIcon::ResizeNorthWest),
                (_, true, true, _) => (Some(ResizeDirection::NorthEast), CursorIcon::ResizeNorthEast),
                (true, _, _, true) => (Some(ResizeDirection::SouthWest), CursorIcon::ResizeSouthWest),
                (_, true, _, true) => (Some(ResizeDirection::SouthEast), CursorIcon::ResizeSouthEast),
                (true, _, _, _) => (Some(ResizeDirection::West), CursorIcon::ResizeWest),
                (_, true, _, _) => (Some(ResizeDirection::East), CursorIcon::ResizeEast),
                (_, _, true, _) => (Some(ResizeDirection::North), CursorIcon::ResizeNorth),
                (_, _, _, true) => (Some(ResizeDirection::South), CursorIcon::ResizeSouth),
                _ => (None, CursorIcon::Default),
            };

            if let Some(d) = dir {
                ctx.set_cursor_icon(cursor);
                if pointer_down && ctx.input(|i| i.pointer.any_pressed()) {
                    self.is_resizing_window = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(d));
                }
            }
        }

        self.is_resizing_window || is_on_border
    }
}

impl eframe::App for PoseaMixerApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // 1. Sistem Tepsisi (Tray) Olaylarını Kontrol Et
        if let Some(ref tray) = self.tray_manager {
            match tray.check_events() {
                TrayAction::ToggleShow => {
                    self.is_window_visible = !self.is_window_visible;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(self.is_window_visible));
                    if self.is_window_visible {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    }
                }
                TrayAction::ToggleBypass => {
                    let mut s = self.settings.write();
                    s.enabled = !s.enabled;
                    s.version += 1;
                }
                TrayAction::Quit => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    std::process::exit(0);
                }
                TrayAction::None => {}
            }
        }

        // Pencere gizliyken dahi tepsi olaylarını yakalamak için hafif polling iste
        if !self.is_window_visible {
            ctx.request_repaint_after(Duration::from_millis(50));
            return;
        }

        // 2. Pencere Kapatma (X Butonu) Yakalama
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.minimize_to_tray {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.is_window_visible = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                ctx.request_repaint_after(Duration::from_millis(50));
                return;
            } else {
                std::process::exit(0);
            }
        }

        let full_rect = ui.ctx().viewport_rect();
        let is_resizing_or_border = self.handle_window_resizing(&ctx, full_rect);

        // Ana Konteyner: Pencerenin TÜM genişlik ve yüksekliğini doldurur (Asla siyah boşluk kalmaz)
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Theme::BG_DARK)
                    .stroke(Stroke::new(1.0, Color32::from_rgb(34, 36, 46)))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::same(14)),
            )
            .show(ui, |ui| {
                let total_w = ui.available_width();
                let total_h = ui.available_height();

                // --- 1. ÜST BAŞLIK ÇUBUĞU (Özel Sürüklenebilir Frameless Titlebar) ---
                let titlebar_height = 36.0;
                let (titlebar_rect, _) = ui.allocate_exact_size(Vec2::new(total_w, titlebar_height), Sense::hover());
                let tb_painter = ui.painter_at(titlebar_rect);

                // Sol: Resmi 5-Bant Posea Audio Ekolayzır Logosu
                let logo_x = titlebar_rect.min.x + 6.0;
                let logo_y = titlebar_rect.center().y;

                // 5 Coral Crimson Çizgi Logo (Resmi Simge ve Sistem Tepsisi ile 100% Uyumlu)
                tb_painter.line_segment([Pos2::new(logo_x, logo_y + 3.5), Pos2::new(logo_x, logo_y - 3.5)], Stroke::new(2.2, Color32::from_rgb(216, 30, 72)));
                tb_painter.line_segment([Pos2::new(logo_x + 4.0, logo_y + 6.5), Pos2::new(logo_x + 4.0, logo_y - 6.5)], Stroke::new(2.2, Color32::from_rgb(255, 53, 98)));
                tb_painter.line_segment([Pos2::new(logo_x + 8.0, logo_y + 9.5), Pos2::new(logo_x + 8.0, logo_y - 9.5)], Stroke::new(2.4, Theme::ACCENT_RED));
                tb_painter.line_segment([Pos2::new(logo_x + 12.0, logo_y + 6.5), Pos2::new(logo_x + 12.0, logo_y - 6.5)], Stroke::new(2.2, Color32::from_rgb(255, 53, 98)));
                tb_painter.line_segment([Pos2::new(logo_x + 16.0, logo_y + 3.5), Pos2::new(logo_x + 16.0, logo_y - 3.5)], Stroke::new(2.2, Color32::from_rgb(216, 30, 72)));

                // POSEA AUDIO Başlığı
                tb_painter.text(
                    Pos2::new(logo_x + 25.0, logo_y),
                    egui::Align2::LEFT_CENTER,
                    "Posea Audio",
                    egui::FontId::proportional(15.5),
                    Color32::WHITE,
                );

                // Hamburger Menü Butonu (☰)
                let hamburger_rect = Rect::from_min_size(Pos2::new(logo_x + 120.0, logo_y - 12.0), Vec2::new(26.0, 24.0));
                let hamburger_resp = ui.interact(hamburger_rect, ui.make_persistent_id("hamburger_btn"), Sense::click());
                let hb_hovered = hamburger_resp.hovered();
                if hb_hovered {
                    tb_painter.rect_filled(hamburger_rect, 4.0, Theme::BG_CARD_HOVER);
                }
                let hb_color = if hb_hovered { Color32::WHITE } else { Color32::from_rgb(140, 144, 160) };
                let hx1 = hamburger_rect.min.x + 5.0;
                let hx2 = hamburger_rect.max.x - 5.0;
                let hy = hamburger_rect.center().y;
                tb_painter.line_segment([Pos2::new(hx1, hy - 5.0), Pos2::new(hx2, hy - 5.0)], Stroke::new(2.0, hb_color));
                tb_painter.line_segment([Pos2::new(hx1, hy),       Pos2::new(hx2, hy)],       Stroke::new(2.0, hb_color));
                tb_painter.line_segment([Pos2::new(hx1, hy + 5.0), Pos2::new(hx2, hy + 5.0)], Stroke::new(2.0, hb_color));
                if hamburger_resp.on_hover_text("Ayarlar & Tercihler").clicked() {
                    self.show_preferences_modal = true;
                }

                // Sağ Taraf: Güç Butonu ve Pencere Kontrolleri
                let right_end = titlebar_rect.max.x;

                // ✕ Kapat Butonu
                let close_rect = Rect::from_min_size(Pos2::new(right_end - 26.0, logo_y - 12.0), Vec2::new(24.0, 24.0));
                let close_resp = ui.interact(close_rect, ui.make_persistent_id("win_close_btn"), Sense::click());
                if close_resp.hovered() {
                    tb_painter.rect_filled(close_rect, 4.0, Color32::from_rgb(180, 30, 40));
                }
                tb_painter.text(close_rect.center(), egui::Align2::CENTER_CENTER, "✕", egui::FontId::proportional(12.0), Color32::WHITE);
                if close_resp.clicked() {
                    if self.minimize_to_tray {
                        self.is_window_visible = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                    } else {
                        std::process::exit(0);
                    }
                }

                // 🗖 / ❐ Boyutlandırma (Büyüt / Geri Yükle) Butonu
                let is_max = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                let max_rect = Rect::from_min_size(Pos2::new(right_end - 54.0, logo_y - 12.0), Vec2::new(24.0, 24.0));
                let max_resp = ui.interact(max_rect, ui.make_persistent_id("win_max_btn"), Sense::click());
                if max_resp.hovered() {
                    tb_painter.rect_filled(max_rect, 4.0, Theme::BG_CARD_HOVER);
                }
                let max_icon = if is_max { "❐" } else { "🗖" };
                tb_painter.text(max_rect.center(), egui::Align2::CENTER_CENTER, max_icon, egui::FontId::proportional(12.0), Color32::WHITE);
                if max_resp.on_hover_text(if is_max { "Geri Yükle" } else { "Büyüt" }).clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_max));
                }

                // — Küçült Butonu
                let min_rect = Rect::from_min_size(Pos2::new(right_end - 82.0, logo_y - 12.0), Vec2::new(24.0, 24.0));
                let min_resp = ui.interact(min_rect, ui.make_persistent_id("win_min_btn"), Sense::click());
                if min_resp.hovered() {
                    tb_painter.rect_filled(min_rect, 4.0, Theme::BG_CARD_HOVER);
                }
                tb_painter.text(min_rect.center(), egui::Align2::CENTER_CENTER, "—", egui::FontId::proportional(12.0), Color32::WHITE);
                if min_resp.clicked() {
                    self.is_window_visible = false;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                }

                // ⏻ Kusursuz IEEE 1621 Güç Butonu (Saat 12 yönü açık daire ve ortadan inen çubuk)
                let pwr_center = Pos2::new(right_end - 120.0, logo_y);
                let pwr_rect = Rect::from_center_size(pwr_center, Vec2::new(32.0, 32.0));
                let pwr_resp = ui.interact(pwr_rect, ui.make_persistent_id("main_power_btn"), Sense::click());
                let is_dsp_enabled = self.settings.read().enabled;

                let (p_fill, p_stroke_color, icon_color) = if is_dsp_enabled {
                    if pwr_resp.hovered() {
                        (Color32::from_rgb(80, 20, 35), Theme::ACCENT_RED_HOVER, Color32::WHITE)
                    } else {
                        (Color32::from_rgb(50, 16, 26), Theme::ACCENT_RED, Theme::ACCENT_RED)
                    }
                } else {
                    (Color32::from_rgb(22, 24, 32), Color32::from_rgb(60, 64, 78), Color32::from_rgb(110, 115, 130))
                };

                tb_painter.circle_filled(pwr_center, 14.5, p_fill);
                tb_painter.circle_stroke(pwr_center, 14.5, Stroke::new(1.8, p_stroke_color));

                let p_r: f32 = 7.0;
                let mut arc_pts = Vec::with_capacity(30);
                let mut deg: f32 = -60.0;
                while deg <= 240.0 {
                    let rad = deg.to_radians();
                    arc_pts.push(Pos2::new(pwr_center.x + p_r * rad.cos(), pwr_center.y + p_r * rad.sin()));
                    deg += 15.0;
                }
                for w in arc_pts.windows(2) {
                    tb_painter.line_segment([w[0], w[1]], Stroke::new(2.2, icon_color));
                }
                tb_painter.line_segment(
                    [Pos2::new(pwr_center.x, pwr_center.y - 7.5), Pos2::new(pwr_center.x, pwr_center.y + 0.5)],
                    Stroke::new(2.2, icon_color),
                );

                if pwr_resp.on_hover_text(if is_dsp_enabled { "DSP Açık (Kapatmak için tıklayın)" } else { "Bypass (Açmak için tıklayın)" }).clicked() {
                    let mut s = self.settings.write();
                    s.enabled = !s.enabled;
                    s.version += 1;
                }

                // Orta Boş Alan: Pencereyi Sürükleme ve Çift Tıklama ile Büyütme
                let drag_rect = Rect::from_min_max(
                    Pos2::new(logo_x + 145.0, titlebar_rect.min.y),
                    Pos2::new(right_end - 145.0, titlebar_rect.max.y),
                );
                let drag_resp = ui.interact(drag_rect, ui.make_persistent_id("win_drag_area"), Sense::click_and_drag());
                if drag_resp.double_clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_max));
                } else if drag_resp.dragged() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                ui.add_space(6.0);

                // --- 2. SEÇİCİLER SATIRI (Preset & Aygıt Seçim Hap Butonları) ---
                let col_spacing = 12.0;
                let half_w = (total_w - col_spacing) * 0.5;

                let mut open_preset_menu_pos = None;
                let mut open_device_menu_pos = None;

                ui.horizontal(|ui| {
                    // Sol Hap: Preset Seçici Buton
                    let (preset_rect, preset_resp) = ui.allocate_exact_size(Vec2::new(half_w, 38.0), Sense::click());
                    let p_bg = if preset_resp.hovered() { Theme::BG_CARD_HOVER } else { Theme::BG_CARD };
                    ui.painter().rect_filled(preset_rect, 8.0, p_bg);
                    ui.painter().rect_stroke(preset_rect, 8.0, Stroke::new(1.0, Color32::from_rgb(34, 36, 48)), egui::StrokeKind::Inside);

                    ui.painter().text(
                        Pos2::new(preset_rect.min.x + 14.0, preset_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &self.selected_preset_name,
                        egui::FontId::proportional(14.0),
                        Color32::WHITE,
                    );

                    ui.painter().text(
                        Pos2::new(preset_rect.max.x - 16.0, preset_rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        "⌵",
                        egui::FontId::proportional(14.0),
                        Theme::ACCENT_RED,
                    );

                    if preset_resp.clicked() {
                        self.preset_dropdown_open = !self.preset_dropdown_open;
                        self.device_dropdown_open = false;
                    }
                    if self.preset_dropdown_open {
                        open_preset_menu_pos = Some(Pos2::new(preset_rect.min.x, preset_rect.max.y + 4.0));
                    }

                    ui.add_space(col_spacing);

                    // Sağ Hap: Aygıt Seçici Buton
                    let (dev_rect, dev_resp) = ui.allocate_exact_size(Vec2::new(half_w, 38.0), Sense::click());
                    let d_bg = if dev_resp.hovered() { Theme::BG_CARD_HOVER } else { Theme::BG_CARD };
                    ui.painter().rect_filled(dev_rect, 8.0, d_bg);
                    ui.painter().rect_stroke(dev_rect, 8.0, Stroke::new(1.0, Color32::from_rgb(34, 36, 48)), egui::StrokeKind::Inside);

                    let cur_out = self.selected_output.read().clone().unwrap_or_else(|| "Otomatik Donanım Hoparlörü".to_string());
                    let display_name = if cur_out.len() > 40 { format!("{}...", &cur_out[..37]) } else { cur_out.clone() };

                    ui.painter().text(
                        Pos2::new(dev_rect.min.x + 14.0, dev_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        format!("🔊  {}", display_name),
                        egui::FontId::proportional(13.0),
                        Color32::from_rgb(220, 222, 230),
                    );

                    ui.painter().text(
                        Pos2::new(dev_rect.max.x - 16.0, dev_rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        "⌵",
                        egui::FontId::proportional(14.0),
                        Theme::ACCENT_RED,
                    );

                    if dev_resp.clicked() {
                        self.device_dropdown_open = !self.device_dropdown_open;
                        self.preset_dropdown_open = false;
                    }
                    if self.device_dropdown_open {
                        open_device_menu_pos = Some(Pos2::new(dev_rect.min.x, dev_rect.max.y + 4.0));
                    }
                });

                ui.add_space(8.0);

                // --- 3. MERKEZ DALGA BİPOLAR GÖRSELLEŞTİRİCİ ---
                let viz_h = if total_h > 580.0 { 100.0 } else if total_h > 510.0 { 90.0 } else { 78.0 };
                VisualizerWidget::show(ui, &self.visualizer, is_dsp_enabled, viz_h);

                ui.add_space(8.0);

                // --- 4. ALT BÖLÜM: 5 KAYDIRICI & 31-BANT GRAPHIC EQUALIZER ---
                // ui.available_height() o anda kalan gerçek piksel yüksekliğini verir (Asla taşmaz)
                let bottom_h = (ui.available_height() - 6.0).max(180.0);
                let left_col_w = (total_w * 0.28).clamp(240.0, 290.0);
                let right_col_w = (total_w - left_col_w - 14.0).max(380.0);

                ui.horizontal_top(|ui| {
                    // --- SOL SÜTUN: 5 Ana Geliştirme Kaydırıcısı (Dikey Orantılı Responsive Boşluk) ---
                    ui.allocate_ui_with_layout(
                        Vec2::new(left_col_w, bottom_h),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.spacing_mut().item_spacing = Vec2::ZERO;

                            let mut s = self.settings.write();
                            let mut changed = false;

                            let slider_h = 32.0;
                            let total_sliders_h = 5.0 * slider_h;
                            let available_gap_space = (bottom_h - total_sliders_h).max(0.0);
                            let gap = (available_gap_space / 4.0).clamp(2.0, 26.0);

                            if Self::render_fx_slider(ui, "Netlik", &mut s.clarity, is_resizing_or_border) { changed = true; }
                            ui.add_space(gap);
                            if Self::render_fx_slider(ui, "Ortam", &mut s.ambience, is_resizing_or_border) { changed = true; }
                            ui.add_space(gap);
                            if Self::render_fx_slider(ui, "Çevresel Ses", &mut s.surround, is_resizing_or_border) { changed = true; }
                            ui.add_space(gap);
                            if Self::render_fx_slider(ui, "Dinamik Güçlendirme", &mut s.dynamic_boost, is_resizing_or_border) { changed = true; }
                            ui.add_space(gap);
                            if Self::render_fx_slider(ui, "Bass Güçlendirme", &mut s.bass_boost, is_resizing_or_border) { changed = true; }

                            if changed {
                                s.version += 1;
                            }
                        },
                    );

                    ui.add_space(14.0);

                    // --- SAĞ SÜTUN: 31-Bant Parametrik ISO Grafik Ekolayzır (Dikey Boyutla Birlikte Büyür) ---
                    ui.allocate_ui_with_layout(
                        Vec2::new(right_col_w, bottom_h),
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            let mut s = self.settings.write();
                            if EqWidget::show(ui, &mut s.eq_bands, bottom_h, &mut self.active_eq_band, is_resizing_or_border) {
                                s.version += 1;
                            }
                        },
                    );
                });

                // --- 5. ÖZEL PRESET AÇILIR MENÜSÜ ---
                if let Some(pos) = open_preset_menu_pos {
                    egui::Area::new(ui.make_persistent_id("preset_popup_area"))
                        .order(egui::Order::Foreground)
                        .fixed_pos(pos)
                        .show(ui.ctx(), |ui| {
                            egui::Frame::new()
                                .fill(Color32::from_rgb(10, 11, 15))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(52, 56, 72)))
                                .corner_radius(CornerRadius::same(6))
                                .inner_margin(Margin::symmetric(0, 4))
                                .show(ui, |ui| {
                                    ui.set_width(half_w);
                                    let presets = [
                                        ("General", "general"),
                                        ("Movies", "movies"),
                                        ("Voice", "voice"),
                                        ("Music", "music"),
                                        ("Gaming", "gaming"),
                                        ("Volume Boost", "volume_boost"),
                                        ("Bass Boost", "bass_boost"),
                                        ("Streaming Video", "streaming"),
                                        ("Default (Flat)", "default"),
                                    ];

                                    for (name, key) in presets {
                                        let is_active = self.selected_preset_name == name;
                                        let (item_rect, item_resp) = ui.allocate_exact_size(Vec2::new(half_w, 32.0), Sense::click());

                                        let bg = if is_active {
                                            Theme::ACCENT_RED
                                        } else if item_resp.hovered() {
                                            Color32::from_rgb(26, 28, 36)
                                        } else {
                                            Color32::TRANSPARENT
                                        };

                                        ui.painter().rect_filled(item_rect, 0.0, bg);

                                        let text_x = if is_active { item_rect.min.x + 32.0 } else { item_rect.min.x + 16.0 };
                                        if is_active {
                                            ui.painter().text(
                                                Pos2::new(item_rect.min.x + 14.0, item_rect.center().y),
                                                egui::Align2::LEFT_CENTER,
                                                "✓",
                                                egui::FontId::proportional(14.0),
                                                Color32::WHITE,
                                            );
                                        }

                                        ui.painter().text(
                                            Pos2::new(text_x, item_rect.center().y),
                                            egui::Align2::LEFT_CENTER,
                                            name,
                                            egui::FontId::proportional(13.5),
                                            Color32::WHITE,
                                        );

                                        if item_resp.clicked() {
                                            self.selected_preset_name = name.to_string();
                                            self.preset_dropdown_open = false;
                                            let mut s = self.settings.write();
                                            match key {
                                                "general" => s.preset_general(),
                                                "movies" => s.preset_movies(),
                                                "voice" => s.preset_voice(),
                                                "music" => s.preset_music(),
                                                "gaming" => s.preset_gaming(),
                                                "volume_boost" => s.preset_volume_boost(),
                                                "bass_boost" => s.preset_bass_boost(),
                                                "streaming" => s.preset_streaming(),
                                                "default" => s.preset_default(),
                                                _ => {}
                                            }
                                        }
                                    }
                                });
                        });
                }

                // --- 6. ÖZEL AYGIT AÇILIR MENÜSÜ ---
                if let Some(pos) = open_device_menu_pos {
                    egui::Area::new(ui.make_persistent_id("device_popup_area"))
                        .order(egui::Order::Foreground)
                        .fixed_pos(pos)
                        .show(ui.ctx(), |ui| {
                            egui::Frame::new()
                                .fill(Color32::from_rgb(10, 11, 15))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(52, 56, 72)))
                                .corner_radius(CornerRadius::same(6))
                                .inner_margin(Margin::symmetric(0, 4))
                                .show(ui, |ui| {
                                    ui.set_width(half_w);
                                    let cur_out = self.selected_output.read().clone().unwrap_or_default();

                                    for dev in &self.cached_devices {
                                        let is_active = cur_out == *dev;
                                        let (item_rect, item_resp) = ui.allocate_exact_size(Vec2::new(half_w, 32.0), Sense::click());

                                        let bg = if is_active {
                                            Theme::ACCENT_RED
                                        } else if item_resp.hovered() {
                                            Color32::from_rgb(26, 28, 36)
                                        } else {
                                            Color32::TRANSPARENT
                                        };

                                        ui.painter().rect_filled(item_rect, 0.0, bg);

                                        let text_x = if is_active { item_rect.min.x + 32.0 } else { item_rect.min.x + 16.0 };
                                        if is_active {
                                            ui.painter().text(
                                                Pos2::new(item_rect.min.x + 14.0, item_rect.center().y),
                                                egui::Align2::LEFT_CENTER,
                                                "✓",
                                                egui::FontId::proportional(14.0),
                                                Color32::WHITE,
                                            );
                                        }

                                        let display_item = if dev.len() > 38 { format!("{}...", &dev[..35]) } else { dev.clone() };
                                        ui.painter().text(
                                            Pos2::new(text_x, item_rect.center().y),
                                            egui::Align2::LEFT_CENTER,
                                            display_item,
                                            egui::FontId::proportional(13.0),
                                            Color32::WHITE,
                                        );

                                        if item_resp.clicked() {
                                            *self.selected_output.write() = Some(dev.clone());
                                            self.device_dropdown_open = false;
                                            self.reload_requested.store(true, Ordering::SeqCst);
                                        }
                                    }
                                });
                        });
                }
            });

        // --- 7. TERCİHLER MODAL DİYALOĞU (FxSound media_1793383686130_286d1f53.png Birebir Tasarımı) ---
        if self.show_preferences_modal {
            egui::Window::new("preferences_modal")
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .fixed_size([560.0, 410.0])
                .frame(
                    egui::Frame::new()
                        .fill(Color32::from_rgb(18, 19, 25))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 48, 62)))
                        .corner_radius(CornerRadius::same(12))
                        .inner_margin(Margin::same(18)),
                )
                .show(ui.ctx(), |ui| {
                    // Modal Başlığı
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("ılı.").size(18.0).strong().color(Theme::ACCENT_RED));
                        ui.label(RichText::new("Tercihler").size(16.0).strong().color(Color32::WHITE));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (c_rect, c_resp) = ui.allocate_exact_size(Vec2::new(26.0, 26.0), Sense::click());
                            if c_resp.hovered() {
                                ui.painter().rect_filled(c_rect, 4.0, Theme::BG_CARD_HOVER);
                            }
                            ui.painter().text(c_rect.center(), egui::Align2::CENTER_CENTER, "✕", egui::FontId::proportional(14.0), Color32::WHITE);
                            if c_resp.clicked() {
                                self.show_preferences_modal = false;
                            }
                        });
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // 2-Sütunlu Gövde: Sol Menü & Sağ İçerik
                    ui.horizontal(|ui| {
                        // Sol Navigasyon Menüsü
                        ui.allocate_ui_with_layout(Vec2::new(115.0, 340.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            let mut nav_item = |ui: &mut egui::Ui, label: &str, tab: PreferencesTab| {
                                let is_sel = self.pref_active_tab == tab;
                                let color = if is_sel { Color32::WHITE } else { Color32::from_rgb(160, 164, 178) };
                                let bg = if is_sel { Color32::from_rgb(34, 37, 48) } else { Color32::TRANSPARENT };
                                if ui.add_sized([110.0, 34.0], egui::Button::new(RichText::new(label).size(13.5).strong().color(color)).fill(bg)).clicked() {
                                    self.pref_active_tab = tab;
                                }
                                ui.add_space(4.0);
                            };

                            nav_item(ui, "🔊  Ses", PreferencesTab::Audio);
                            nav_item(ui, "⚙  Genel", PreferencesTab::General);
                            nav_item(ui, "💬  Yardım", PreferencesTab::Help);
                        });

                        ui.separator();
                        ui.add_space(8.0);

                        // Sağ İçerik Alanı (FxSound Birebir Satır Satır Düzen)
                        ui.allocate_ui_with_layout(Vec2::new(395.0, 340.0), egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);

                            match self.pref_active_tab {
                                PreferencesTab::Audio => {
                                    ui.label(RichText::new("Ses").size(15.0).strong().color(Color32::WHITE));
                                    ui.add_space(2.0);

                                    // 1. Tercih Edilen Çıkış
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Tercih edilen çıkış:").size(12.0).color(Color32::from_rgb(170, 174, 188)));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            let cur_out = self.selected_output.read().clone().unwrap_or_else(|| "Otomatik".to_string());
                                            let display_dev = if cur_out.len() > 28 { format!("{}...", &cur_out[..25]) } else { cur_out.clone() };
                                            egui::ComboBox::from_id_salt("pref_modal_out_box")
                                                .selected_text(display_dev)
                                                .width(230.0)
                                                .show_ui(ui, |ui| {
                                                    for d in &self.cached_devices {
                                                        if ui.selectable_label(cur_out == *d, d).clicked() {
                                                            *self.selected_output.write() = Some(d.clone());
                                                            self.reload_requested.store(true, Ordering::SeqCst);
                                                        }
                                                    }
                                                });
                                        });
                                    });

                                    // 2. Ekolayzır: 31 bant
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Ekolayzır:").size(12.0).color(Color32::from_rgb(170, 174, 188)));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            let (e_rect, _) = ui.allocate_exact_size(Vec2::new(230.0, 26.0), Sense::hover());
                                            ui.painter().rect_filled(e_rect, 4.0, Color32::from_rgb(14, 15, 20));
                                            ui.painter().rect_stroke(e_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(38, 40, 52)), egui::StrokeKind::Inside);
                                            ui.painter().text(Pos2::new(e_rect.min.x + 10.0, e_rect.center().y), egui::Align2::LEFT_CENTER, "31 bant", egui::FontId::proportional(12.5), Color32::WHITE);
                                            ui.painter().text(Pos2::new(e_rect.max.x - 12.0, e_rect.center().y), egui::Align2::RIGHT_CENTER, "⌵", egui::FontId::proportional(12.0), Theme::ACCENT_RED);
                                        });
                                    });

                                    // 3. Ana Ses Seviyesi Slider
                                    let mut s = self.settings.write();
                                    let vol_str = format!("{:.0} dB", self.master_volume_db);
                                    if Self::render_pref_slider(ui, "Ana ses seviyesi:", &mut self.master_volume_db, -12.0, 12.0, &vol_str) {
                                        s.master_volume = 10.0_f32.powf(self.master_volume_db / 20.0);
                                        s.version += 1;
                                    }

                                    // 4. Normalizasyon Slider
                                    let norm_str = format!("{:.0} dB", self.normalization_db);
                                    Self::render_pref_slider(ui, "Normalizasyon:", &mut self.normalization_db, 0.0, 12.0, &norm_str);

                                    // 5. Filtre Q Slider
                                    let q_str = format!("{:.1}x", self.filter_q);
                                    Self::render_pref_slider(ui, "Filtre Q:", &mut self.filter_q, 0.5, 3.0, &q_str);

                                    // 6. Denge (Sol - Sağ) Slider
                                    let bal_str = format!("{:.0} dB", self.balance * 6.0);
                                    Self::render_pref_slider(ui, "Denge:", &mut self.balance, -1.0, 1.0, &bal_str);

                                    // Denge Altı Sol/Sağ Etiketleri (FxSound media_1793383686130_286d1f53.png Birebir)
                                    let (sub_rect, _) = ui.allocate_exact_size(Vec2::new(390.0, 12.0), Sense::hover());
                                    let track_x1 = sub_rect.min.x + 130.0;
                                    let track_x2 = sub_rect.max.x - 48.0;
                                    ui.painter().text(Pos2::new(track_x1, sub_rect.center().y), egui::Align2::LEFT_CENTER, "Sol", egui::FontId::proportional(9.0), Color32::from_rgb(110, 114, 130));
                                    ui.painter().text(Pos2::new(track_x2, sub_rect.center().y), egui::Align2::RIGHT_CENTER, "Sağ", egui::FontId::proportional(9.0), Color32::from_rgb(110, 114, 130));

                                    ui.add_space(8.0);

                                    // 7. İki Büyük Aksiyon Butonu
                                    if ui.add_sized([390.0, 32.0], egui::Button::new(RichText::new("Varsayılanları geri yükle").size(13.0).strong().color(Color32::WHITE)).fill(Theme::ACCENT_RED)).clicked() {
                                        self.master_volume_db = 0.0;
                                        self.normalization_db = 0.0;
                                        self.filter_q = 1.0;
                                        self.balance = 0.0;
                                        s.master_volume = 1.0;
                                        s.version += 1;
                                    }

                                    ui.add_space(4.0);

                                    if ui.add_sized([390.0, 32.0], egui::Button::new(RichText::new("Ön Ayarları fabrika ayarlara sıfırlayın").size(13.0).color(Color32::WHITE)).fill(Color32::from_rgb(120, 32, 46))).clicked() {
                                        s.preset_default();
                                        self.selected_preset_name = "Default (Flat)".to_string();
                                    }
                                }
                                PreferencesTab::General => {
                                    ui.label(RichText::new("Genel Tercihler").size(15.0).strong().color(Color32::WHITE));
                                    ui.add_space(12.0);

                                    if ui.checkbox(&mut self.autostart_enabled, "Windows açılışında otomatik başlat").changed() {
                                        let _ = autostart::set_autostart(self.autostart_enabled);
                                    }

                                    ui.add_space(8.0);

                                    ui.checkbox(&mut self.minimize_to_tray, "Pencere kapatıldığında sistem tepsisine küçült");

                                    ui.add_space(16.0);

                                    if ui.button("🔄 Ses Aygıtlarını Yeniden Tara").clicked() {
                                        self.cached_devices = Self::scan_devices();
                                        self.reload_requested.store(true, Ordering::SeqCst);
                                    }
                                }
                                PreferencesTab::Help => {
                                    ui.label(RichText::new("Posea Audio Hakkında").size(15.0).strong().color(Color32::WHITE));
                                    ui.add_space(8.0);
                                    ui.label(RichText::new("Sürüm: 1.0.0 (Release)").color(Theme::TEXT_SECONDARY));
                                    ui.label(RichText::new("Motor: Posea Virtual Driver & Lock-Free DSP").color(Theme::TEXT_SECONDARY));
                                    ui.label(RichText::new("Ekolayzır: 31-Bant 1/3 Octave ISO").color(Theme::TEXT_SECONDARY));
                                    ui.add_space(10.0);
                                    ui.label(RichText::new("Sanal Aygıt Durumu: Aktif (Hoparlör - Posea Audio Virtual Device)").color(Theme::ACCENT_GREEN));
                                }
                            }
                        });
                    });
                });
        }
    }
}
