#![cfg_attr(windows, windows_subsystem = "windows")]

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
use crate::utils::config::AppConfig;
use crate::utils::single_instance::SingleInstance;

fn main() -> eframe::Result<()> {
    std::panic::set_hook(Box::new(|info| {
        let _ = std::fs::write("panic.txt", format!("{:#?}", info));
    }));

    let is_minimized = std::env::args().any(|arg| arg == "--minimized");

    // Tek Çalışan Örnek (Single Instance) Koruması:
    // Zaten bir kopya çalışıyorsa, ona pencereyi açmasını söyler ve bu süreci sonlandırır.
    let single_instance = SingleInstance::acquire(is_minimized);

    // Kayıtlı kullanıcı ayarlarını yükle (AppData/PoseaAudio/config.json)
    let config = AppConfig::load();

    let mut initial_dsp = DspSettings::default();
    initial_dsp.enabled = config.dsp.enabled;
    initial_dsp.clarity = config.dsp.clarity;
    initial_dsp.ambience = config.dsp.ambience;
    initial_dsp.surround = config.dsp.surround;
    initial_dsp.dynamic_boost = config.dsp.dynamic_boost;
    initial_dsp.bass_boost = config.dsp.bass_boost;
    initial_dsp.master_volume = config.dsp.master_volume;
    initial_dsp.eq_bands = config.dsp.eq_bands;

    let settings = Arc::new(RwLock::new(initial_dsp));
    let visualizer = VisualizerData::new();
    let selected_input = Arc::new(RwLock::new(None));
    let selected_output = Arc::new(RwLock::new(config.ui.selected_output.clone()));
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

    // Çerçevesiz (Frameless) ve Şeffaf Köşeli Modern Pencere
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Posea Audio")
        .with_decorations(false)         // Standart Windows başlık çubuğunu kaldırır
        .with_transparent(true)           // Yuvarlatılmış zarif köşeler için şeffaflık
        .with_resizable(true)             // Boyutlandırılabilir pencere
        .with_inner_size([960.0, 540.0])  // 16:9 geniş format oranı
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
                config,
                single_instance,
            );
            Ok(Box::new(app))
        }),
    );

    if let Err(ref e) = res {
        let _ = std::fs::write("err.txt", format!("{:#?}", e));
    }
    res
}
