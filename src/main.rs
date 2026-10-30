#![windows_subsystem = "windows"]

mod dsp;
mod engine;
mod gui;
mod utils;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use eframe::egui;
use parking_lot::RwLock;

use crate::dsp::{DspSettings, VisualizerData};
use crate::engine::AudioBridge;
use crate::gui::app::PoseaMixerApp;

fn main() -> eframe::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        let _ = std::fs::write("panic.txt", format!("{:#?}", info));
    }));

    let is_minimized = std::env::args().any(|arg| arg == "--minimized");

    let settings = Arc::new(RwLock::new(DspSettings::default()));
    let visualizer = VisualizerData::new();
    let selected_input = Arc::new(RwLock::new(None));
    let selected_output = Arc::new(RwLock::new(None));
    let reload_requested = Arc::new(AtomicBool::new(false));

    // Ses köprüsünü (Audio Bridge) bağımsız yüksek öncelikli iş parçacığında başlat
    let bridge = AudioBridge::new(
        settings.clone(),
        selected_input.clone(),
        selected_output.clone(),
        reload_requested.clone(),
        visualizer.clone(),
    );

    let _bridge_thread = std::thread::Builder::new()
        .name("posea-audio-bridge".to_string())
        .spawn(move || {
            bridge.run();
        });

    // Resmi Posea Audio Simgesini Bellekten Yükle (Görev Çubuğu ve Pencere İçin)
    let icon_data = match image::load_from_memory(include_bytes!("../assets/icon_256.png")) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            Some(egui::IconData {
                rgba: rgba.into_raw(),
                width: w,
                height: h,
            })
        }
        Err(_) => None,
    };

    // FxSound Birebir Eşleşmesi İçin Çerçevesiz (Frameless) ve Şeffaf Köşeli Pencere
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Posea Audio")
        .with_decorations(false)         // Standart Windows başlık çubuğunu kaldırır
        .with_transparent(true)           // Yuvarlatılmış zarif köşeler için şeffaflık
        .with_resizable(true)             // Boyutlandırılabilir pencere
        .with_inner_size([960.0, 540.0])  // FxSound geniş format oranı
        .with_min_inner_size([880.0, 480.0])
        .with_visible(!is_minimized);

    if let Some(icon) = icon_data {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let res = eframe::run_native(
        "Posea Audio",
        options,
        Box::new(move |cc| {
            let app = PoseaMixerApp::new(
                cc,
                settings,
                visualizer,
                selected_input,
                selected_output,
                reload_requested,
            );
            Ok(Box::new(app))
        }),
    );

    if let Err(ref e) = res {
        let _ = std::fs::write("err.txt", format!("{:#?}", e));
    }
    res
}
