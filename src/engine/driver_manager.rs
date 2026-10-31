use std::process::Command;
use std::path::Path;

pub struct DriverManager;

impl DriverManager {
    /// Sistemde bir sanal ses kartı olup olmadığını kontrol eder
    pub fn is_virtual_device_present(devices: &[String]) -> bool {
        devices.iter().any(|name| {
            name.contains("Posea") 
                || name.contains("Virtual") 
                || name.contains("Cable") 
                
                || name.contains("Steam Streaming")
        })
    }

    /// Windows sürücüsünü kurar (Yönetici yetkisi gerektirir)
    pub fn install_windows_driver(inf_path: &Path) -> Result<String, String> {
        if !inf_path.exists() {
            return Err(format!("INF sürücü dosyası bulunamadı: {:?}", inf_path));
        }

        let output = Command::new("pnputil")
            .arg("/add-driver")
            .arg(inf_path)
            .arg("/install")
            .output()
            .map_err(|e| format!("pnputil çalıştırılamadı: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if output.status.success() {
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(format!("Sürücü kurulum hatası: {}\n{}", stdout, stderr))
        }
    }

    /// Linux PipeWire / PulseAudio için sanal ses kanalını oluşturur
    pub fn setup_linux_null_sink() -> Result<String, String> {
        let output = Command::new("pactl")
            .arg("load-module")
            .arg("module-null-sink")
            .arg("sink_name=posea_audio")
            .arg("sink_properties=device.description=\"Posea Audio Device\"")
            .output()
            .map_err(|e| format!("pactl çalıştırılamadı: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Linux sanal ses kanalını kaldırır
    pub fn remove_linux_null_sink(module_id: &str) -> Result<(), String> {
        let status = Command::new("pactl")
            .arg("unload-module")
            .arg(module_id)
            .status()
            .map_err(|e| format!("pactl çalıştırılamadı: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err("Modül kaldırılamadı".to_string())
        }
    }
}
