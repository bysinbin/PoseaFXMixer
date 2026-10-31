use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    ToggleBypass,    // Ctrl + Alt + Space
    ToggleBassBoost, // Ctrl + Alt + B
    CyclePreset,     // Ctrl + Alt + P
    ToggleMute,      // Ctrl + Alt + M
}

pub struct HotkeyManager {
    rx: Receiver<HotkeyAction>,
}

impl HotkeyManager {
    pub fn new(egui_ctx: Arc<RwLock<Option<eframe::egui::Context>>>) -> Self {
        let (tx, rx) = channel();

        #[cfg(windows)]
        {
            std::thread::Builder::new()
                .name("posea-global-hotkeys".to_string())
                .spawn(move || {
                    use windows::Win32::UI::Input::KeyboardAndMouse::{
                        RegisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
                        VK_SPACE,
                    };
                    use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

                    let mods = HOT_KEY_MODIFIERS(MOD_CONTROL.0 | MOD_ALT.0 | MOD_NOREPEAT.0);

                    // 1: Toggle Bypass (Ctrl + Alt + Space)
                    let _ = unsafe { RegisterHotKey(None, 1, mods, VK_SPACE.0 as u32) };
                    // 2: Toggle Bass Boost (Ctrl + Alt + B)
                    let _ = unsafe { RegisterHotKey(None, 2, mods, 0x42) };
                    // 3: Cycle Preset (Ctrl + Alt + P)
                    let _ = unsafe { RegisterHotKey(None, 3, mods, 0x50) };
                    // 4: Toggle Mute (Ctrl + Alt + M)
                    let _ = unsafe { RegisterHotKey(None, 4, mods, 0x4D) };

                    let mut msg = MSG::default();
                    unsafe {
                        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                            if msg.message == WM_HOTKEY {
                                let action = match msg.wParam.0 {
                                    1 => Some(HotkeyAction::ToggleBypass),
                                    2 => Some(HotkeyAction::ToggleBassBoost),
                                    3 => Some(HotkeyAction::CyclePreset),
                                    4 => Some(HotkeyAction::ToggleMute),
                                    _ => None,
                                };

                                if let Some(a) = action {
                                    let _ = tx.send(a);
                                    if let Some(ctx) = egui_ctx.read().as_ref() {
                                        ctx.request_repaint();
                                    }
                                }
                            }
                        }
                    }
                })
                .expect("Failed to spawn hotkey thread");
        }

        Self { rx }
    }

    pub fn poll_action(&self) -> Option<HotkeyAction> {
        self.rx.try_recv().ok()
    }
}
