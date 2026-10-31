/// Windows Uygulama Başına Ses ve Karıştırıcı Yöneticisi (Per-App Audio Session Mixer)

#[derive(Debug, Clone)]
pub struct AppAudioSession {
    pub pid: u32,
    pub name: String,
    pub volume: f32, // 0.0 .. 1.0
    pub is_muted: bool,
}

#[cfg(windows)]
pub mod sys {
    use super::AppAudioSession;
    use windows::core::Interface;
    use windows::Win32::Media::Audio::*;
    use windows::Win32::System::Com::*;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT};

    pub fn enumerate_sessions() -> Vec<AppAudioSession> {
        let mut list = Vec::new();
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            let enumerator: IMMDeviceEnumerator = match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
                Ok(e) => e,
                Err(_) => return list,
            };

            let device = match enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) {
                Ok(d) => d,
                Err(_) => return list,
            };

            let session_manager: IAudioSessionManager2 = match device.Activate(CLSCTX_ALL, None) {
                Ok(m) => m,
                Err(_) => return list,
            };

            let session_enum = match session_manager.GetSessionEnumerator() {
                Ok(e) => e,
                Err(_) => return list,
            };

            let count = session_enum.GetCount().unwrap_or(0);
            for i in 0..count {
                if let Ok(control) = session_enum.GetSession(i) {
                    if let Ok(control2) = control.cast::<IAudioSessionControl2>() {
                        let pid = control2.GetProcessId().unwrap_or(0);
                        if pid == 0 || pid == std::process::id() {
                            continue; // Sistem sesi veya Posea Audio'nun kendisini geç
                        }

                        let volume_control: Result<ISimpleAudioVolume, _> = control.cast();
                        if let Ok(vol) = volume_control {
                            let master_vol = vol.GetMasterVolume().unwrap_or(1.0);
                            let muted = vol.GetMute().map(|m| m.as_bool()).unwrap_or(false);

                            let name = get_process_name_by_pid(pid).unwrap_or_else(|| format!("Uygulama (PID: {})", pid));
                            list.push(AppAudioSession {
                                pid,
                                name,
                                volume: master_vol,
                                is_muted: muted,
                            });
                        }
                    }
                }
            }
        }
        list
    }

    pub fn set_app_volume(pid: u32, volume: f32) {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let enumerator: IMMDeviceEnumerator = match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
                Ok(e) => e,
                Err(_) => return,
            };
            let device = match enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia) {
                Ok(d) => d,
                Err(_) => return,
            };
            let session_manager: IAudioSessionManager2 = match device.Activate(CLSCTX_ALL, None) {
                Ok(m) => m,
                Err(_) => return,
            };
            let session_enum = match session_manager.GetSessionEnumerator() {
                Ok(e) => e,
                Err(_) => return,
            };
            let count = session_enum.GetCount().unwrap_or(0);
            for i in 0..count {
                if let Ok(control) = session_enum.GetSession(i) {
                    if let Ok(control2) = control.cast::<IAudioSessionControl2>() {
                        if control2.GetProcessId().unwrap_or(0) == pid {
                            if let Ok(vol) = control.cast::<ISimpleAudioVolume>() {
                                let _ = vol.SetMasterVolume(volume.clamp(0.0, 1.0), std::ptr::null());
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    fn get_process_name_by_pid(pid: u32) -> Option<String> {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
            let mut buf = [0u16; 512];
            let mut size = buf.len() as u32;
            let res = QueryFullProcessImageNameW(handle, PROCESS_NAME_FORMAT(0), windows::core::PWSTR(buf.as_mut_ptr()), &mut size);
            let _ = CloseHandle(handle);
            if res.is_ok() && size > 0 {
                let full_path = String::from_utf16_lossy(&buf[..size as usize]);
                let file_name = std::path::Path::new(&full_path)
                    .file_name()?
                    .to_string_lossy()
                    .to_string();
                return Some(file_name);
            }
        }
        None
    }
}

#[cfg(not(windows))]
pub mod sys {
    use super::AppAudioSession;
    pub fn enumerate_sessions() -> Vec<AppAudioSession> { Vec::new() }
    pub fn set_app_volume(_pid: u32, _volume: f32) {}
}

pub fn list_active_app_sessions() -> Vec<AppAudioSession> {
    sys::enumerate_sessions()
}

pub fn set_app_volume(pid: u32, volume: f32) {
    sys::set_app_volume(pid, volume);
}
