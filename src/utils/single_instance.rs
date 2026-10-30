use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;

const SINGLE_INSTANCE_PORT: u16 = 48765;

#[cfg(windows)]
use windows::core::w;
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
#[cfg(windows)]
use windows::Win32::System::Threading::CreateMutexW;

pub struct SingleInstanceGuard {
    #[cfg(windows)]
    _mutex_handle: Option<HANDLE>,
    pub show_window_flag: Arc<AtomicBool>,
    pub egui_ctx: Arc<RwLock<Option<eframe::egui::Context>>>,
}

#[cfg(windows)]
impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if let Some(h) = self._mutex_handle {
            unsafe {
                let _ = CloseHandle(h);
            }
        }
    }
}

pub struct SingleInstance;

impl SingleInstance {
    /// Uygulamanın zaten çalışıp çalışmadığını denetler.
    /// Zaten çalışıyorsa:
    /// - Mevcut çalışan örneğe sinyal göndererek pencereyi ön plana getirmesini ister (is_minimized değilse).
    /// - Bu ikinci süreci temiz bir şekilde (exit 0) sonlandırır.
    /// İlk örnek ise:
    /// - Bir SingleInstanceGuard döner; bu guard işlem boyunca canlı tutulmalıdır.
    pub fn acquire(is_minimized: bool) -> SingleInstanceGuard {
        #[cfg(windows)]
        let (already_exists_mutex, mutex_handle) = unsafe {
            let name = w!("Local\\PoseaAudio_SingleInstance_Mutex");
            let handle = CreateMutexW(None, true, name);
            let exists = GetLastError() == ERROR_ALREADY_EXISTS;
            (exists, handle.ok())
        };

        #[cfg(not(windows))]
        let (already_exists_mutex, mutex_handle) = (false, None);

        // Eğer Windows Mutex başka bir sürecin çalıştığını bildiriyorsa:
        if already_exists_mutex {
            Self::notify_existing_and_exit(is_minimized);
        }

        // IPC Portunu dinlemeyi dene
        match TcpListener::bind(("127.0.0.1", SINGLE_INSTANCE_PORT)) {
            Ok(listener) => {
                let show_window_flag = Arc::new(AtomicBool::new(false));
                let egui_ctx: Arc<RwLock<Option<eframe::egui::Context>>> = Arc::new(RwLock::new(None));

                let show_flag_clone = show_window_flag.clone();
                let egui_ctx_clone = egui_ctx.clone();

                std::thread::Builder::new()
                    .name("posea-single-instance-ipc".to_string())
                    .spawn(move || {
                        for stream in listener.incoming() {
                            if let Ok(mut stream) = stream {
                                let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                                let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
                                let mut buf = [0u8; 32];
                                if let Ok(n) = stream.read(&mut buf) {
                                    let msg = &buf[..n];
                                    if msg.starts_with(b"SHOW") {
                                        show_flag_clone.store(true, Ordering::SeqCst);
                                        if let Some(ctx) = egui_ctx_clone.read().as_ref() {
                                            ctx.request_repaint();
                                        }
                                        let _ = stream.write_all(b"OK\n");
                                    } else if msg.starts_with(b"PING") {
                                        let _ = stream.write_all(b"OK\n");
                                    }
                                }
                            }
                        }
                    })
                    .expect("Failed to spawn single instance IPC listener");

                SingleInstanceGuard {
                    #[cfg(windows)]
                    _mutex_handle: mutex_handle,
                    show_window_flag,
                    egui_ctx,
                }
            }
            Err(_) => {
                // Port doluydu (demek ki başka bir örnek çalışıyor)
                Self::notify_existing_and_exit(is_minimized);
            }
        }
    }

    fn notify_existing_and_exit(is_minimized: bool) -> ! {
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_millis(1500) {
            if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", SINGLE_INSTANCE_PORT)) {
                let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
                let cmd = if is_minimized { b"PING\n" } else { b"SHOW\n" };
                let _ = stream.write_all(cmd);
                let mut ack = [0u8; 8];
                let _ = stream.read(&mut ack);
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }

        std::process::exit(0);
    }
}
