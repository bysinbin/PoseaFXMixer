use std::process::Command;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "PoseaFXMixer";

pub fn is_autostart_enabled() -> bool {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("reg");
        cmd.args(["query", RUN_KEY, "/v", APP_NAME]);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        if let Ok(output) = cmd.output() {
            return output.status.success();
        }
    }
    false
}

pub fn set_autostart(enable: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        if enable {
            let exe_path = std::env::current_exe()
                .map_err(|e| format!("Exe yolu bulunamadı: {}", e))?;
            let val = format!("\"{}\" --minimized", exe_path.to_string_lossy());

            let mut cmd = Command::new("reg");
            cmd.args(["add", RUN_KEY, "/v", APP_NAME, "/t", "REG_SZ", "/d", &val, "/f"]);
            cmd.creation_flags(0x08000000);
            let out = cmd.output().map_err(|e| format!("Kayıt girdisi eklenemedi: {}", e))?;
            if !out.status.success() {
                return Err(String::from_utf8_lossy(&out.stderr).to_string());
            }
        } else {
            let mut cmd = Command::new("reg");
            cmd.args(["delete", RUN_KEY, "/v", APP_NAME, "/f"]);
            cmd.creation_flags(0x08000000);
            let _ = cmd.output();
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        Err("Otomatik başlatma yalnızca Windows platformunda desteklenmektedir.".to_string())
    }
}
