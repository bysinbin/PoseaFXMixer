use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::dsp::{DspSettings, NUM_EQ_BANDS};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub dsp: DspConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DspConfig {
    pub enabled: bool,
    pub clarity: f32,
    pub ambience: f32,
    pub surround: f32,
    pub dynamic_boost: f32,
    pub bass_boost: f32,
    pub master_volume: f32,
    pub eq_bands: [f32; NUM_EQ_BANDS],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPreset {
    pub name: String,
    pub clarity: f32,
    pub ambience: f32,
    pub surround: f32,
    pub dynamic_boost: f32,
    pub bass_boost: f32,
    pub eq_bands: [f32; NUM_EQ_BANDS],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub selected_preset_name: String,
    pub selected_output: Option<String>,
    pub minimize_to_tray: bool,
    pub autostart_enabled: bool,
    pub master_volume_db: f32,
    pub normalization_db: f32,
    pub filter_q: f32,
    pub balance: f32,
    #[serde(default)]
    pub is_mini_mode: bool,
    #[serde(default)]
    pub custom_presets: Vec<CustomPreset>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let dsp_def = DspSettings::default();
        Self {
            dsp: DspConfig {
                enabled: dsp_def.enabled,
                clarity: dsp_def.clarity,
                ambience: dsp_def.ambience,
                surround: dsp_def.surround,
                dynamic_boost: dsp_def.dynamic_boost,
                bass_boost: dsp_def.bass_boost,
                master_volume: dsp_def.master_volume,
                eq_bands: dsp_def.eq_bands,
            },
            ui: UiConfig {
                selected_preset_name: "Music".to_string(),
                selected_output: None,
                minimize_to_tray: true,
                autostart_enabled: false,
                master_volume_db: 0.0,
                normalization_db: 0.0,
                filter_q: 1.0,
                balance: 0.0,
                is_mini_mode: false,
                custom_presets: Vec::new(),
            },
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> PathBuf {
        #[cfg(windows)]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                return PathBuf::from(appdata).join("PoseaAudio");
            }
        }
        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home).join("Library/Application Support/PoseaAudio");
            }
        }
        #[cfg(target_os = "linux")]
        {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home).join(".config/posea-audio");
            }
        }
        PathBuf::from(".")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    /// Ayarları diskten yükler, dosya yoksa veya bozuksa varsayılan ayarları döner ve diske kaydeder
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<Self>(&content) {
                    return cfg;
                }
            }
        }
        let def = Self::default();
        def.save();
        def
    }

    /// Ayarları JSON olarak diske kaydeder
    pub fn save(&self) {
        let dir = Self::config_dir();
        let _ = fs::create_dir_all(&dir);
        let path = Self::config_path();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    /// Özel bir ön ayarı .posea/.json dosyasına aktarır
    pub fn export_preset_file(preset: &CustomPreset, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(preset).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }

    /// .posea/.json dosyasından özel ön ayar yükler
    pub fn import_preset_file(path: &Path) -> Result<CustomPreset, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str::<CustomPreset>(&content).map_err(|e| e.to_string())
    }
}
