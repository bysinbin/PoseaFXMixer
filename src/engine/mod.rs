pub mod bridge;
pub mod driver_manager;
pub mod endpoint_volume;
pub use bridge::AudioBridge;
pub use driver_manager::DriverManager;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crate::dsp::effects::DspProcessor;
use crate::dsp::limiter::FastLimiter;
use crate::dsp::DspSettings;

pub struct AudioEngine {
    settings: Arc<parking_lot::RwLock<DspSettings>>,
    running: Arc<AtomicBool>,
}

impl AudioEngine {
    pub fn new(settings: Arc<parking_lot::RwLock<DspSettings>>) -> Self {
        Self {
            settings,
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// RDP ve dinamik aygıt değişikliklerine karşı dayanıklı ana çalışma döngüsü
    pub fn run_resilient_loop(&self) {
        #[cfg(windows)]
        unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            );
        }
        let host = cpal::default_host();
        println!(">>> [PoseaFX Engine] Başlatıldı. Ses Hostu: {:?}", host.id());

        while self.running.load(Ordering::SeqCst) {
            // 1. Aktif çıkış aygıtını bulmayı dene
            let device = match host.default_output_device().or_else(|| {
                host.output_devices().ok().and_then(|mut d| d.next())
            }) {
                Some(dev) => dev,
                None => {
                    // RDP bağlantısı kesildiğinde veya aygıt bulunmadığında buraya düşer
                    eprintln!(">>> [RDP / Endpoint Bekleniyor] Aktif çıkış aygıtı yok (Oturum kilitli veya RDP arka planda). Yeniden deneniyor...");
                    std::thread::sleep(Duration::from_millis(1500));
                    continue;
                }
            };

            let dev_name = device.description()
                .map(|d| d.name().to_string())
                .unwrap_or_else(|_| "Bilinmeyen Aygıt".to_string());

            println!(">>> [Ses Bağlantısı Kuruldu] Hedef Uç Nokta: [{}]", dev_name);

            // 2. Desteklenen konfigürasyonu al
            let config = match device.default_output_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!(">>> Konfigürasyon alınamadı ({}). Alternatif aranıyor...", e);
                    match device.supported_output_configs().ok().and_then(|mut c| c.next()) {
                        Some(supported) => supported.with_max_sample_rate(),
                        None => {
                            std::thread::sleep(Duration::from_millis(1000));
                            continue;
                        }
                    }
                }
            };

            let sample_rate = config.sample_rate() as f32;
            let channels = config.channels() as usize;
            println!(">>> Konfigürasyon: {} Hz, {} Kanal, Format: {:?}", sample_rate, channels, config.sample_format());

            // 3. DSP ve Limiter hazırlığı
            let mut dsp = DspProcessor::new(sample_rate);
            let mut limiter = FastLimiter::new(sample_rate);
            let current_settings = self.settings.read().clone();
            dsp.update_settings(&current_settings);

            // Test sinyal üreteci (Gerçek implementasyonda bu sanal kablo/girişten beslenir)
            let mut clock = 0f32;
            let settings_ref = self.settings.clone();

            let device_lost = Arc::new(AtomicBool::new(false));
            let device_lost_clone = device_lost.clone();

            let err_fn = move |err| {
                eprintln!(">>> [UYARI] Ses akışında hata (RDP bağlantısı kesilmiş olabilir): {}", err);
                device_lost_clone.store(true, Ordering::SeqCst);
            };

            let format = config.sample_format();
            let stream_cfg: cpal::StreamConfig = config.into();

            let stream_result = match format {
                cpal::SampleFormat::F32 => device.build_output_stream(
                    stream_cfg,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        let active_settings = settings_ref.read().clone();
                        for frame in data.chunks_mut(channels) {
                            // 110Hz + 220Hz test zengin akor (müzikal demo)
                            clock = (clock + 1.0) % sample_rate;
                            let raw_sample = ((clock * 220.0 * 2.0 * std::f32::consts::PI / sample_rate).sin() * 0.15)
                                + ((clock * 110.0 * 2.0 * std::f32::consts::PI / sample_rate).sin() * 0.2);

                            // DSP Hattı: EQ -> Bass -> Clarity -> Spatial
                            let (l, r) = dsp.process_stereo(raw_sample, raw_sample, &active_settings);
                            // Koruma: Limiter
                            let (out_l, out_r) = limiter.process_stereo(l, r);

                            if channels >= 2 {
                                frame[0] = out_l;
                                frame[1] = out_r;
                            } else if channels == 1 {
                                frame[0] = 0.5 * (out_l + out_r);
                            }
                        }
                    },
                    err_fn,
                    None,
                ),
                cpal::SampleFormat::I16 => device.build_output_stream(
                    stream_cfg,
                    move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                        let active_settings = settings_ref.read().clone();
                        for frame in data.chunks_mut(channels) {
                            clock = (clock + 1.0) % sample_rate;
                            let raw_sample = ((clock * 220.0 * 2.0 * std::f32::consts::PI / sample_rate).sin() * 0.15)
                                + ((clock * 110.0 * 2.0 * std::f32::consts::PI / sample_rate).sin() * 0.2);

                            let (l, r) = dsp.process_stereo(raw_sample, raw_sample, &active_settings);
                            let (out_l, out_r) = limiter.process_stereo(l, r);

                            let s_l = (out_l * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                            let s_r = (out_r * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;

                            if channels >= 2 {
                                frame[0] = s_l;
                                frame[1] = s_r;
                            } else if channels == 1 {
                                frame[0] = ((s_l as i32 + s_r as i32) / 2) as i16;
                            }
                        }
                    },
                    err_fn,
                    None,
                ),
                unsupported => {
                    eprintln!(">>> Desteklenmeyen format: {:?}", unsupported);
                    std::thread::sleep(Duration::from_millis(1000));
                    continue;
                }
            };

            match stream_result {
                Ok(stream) => {
                    if let Err(e) = stream.play() {
                        eprintln!(">>> Akış başlatılamadı: {}", e);
                        std::thread::sleep(Duration::from_millis(500));
                        continue;
                    }

                    println!(">>> [CANLI] Ses akışı çalınıyor. DSP ve Limiter devrede!");

                    // Cihaz düşene veya kapanana kadar akışı koru
                    while !device_lost.load(Ordering::SeqCst) && self.running.load(Ordering::SeqCst) {
                        std::thread::sleep(Duration::from_millis(250));
                    }

                    println!(">>> [Kapatılıyor] Akış sonlandırılıyor, yeniden bağlanma döngüsüne dönülüyor...");
                    drop(stream);
                }
                Err(e) => {
                    eprintln!(">>> Akış oluşturulamadı ({}). Yeniden taranıyor...", e);
                    std::thread::sleep(Duration::from_millis(1000));
                }
            }
        }
    }
}
