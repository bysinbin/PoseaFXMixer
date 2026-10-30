use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use parking_lot::RwLock;

use crate::dsp::effects::DspProcessor;
use crate::dsp::limiter::FastLimiter;
use crate::dsp::visualizer_analyzer::VisualizerAnalyzer;
use crate::dsp::visualizer_data::VisualizerData;
use crate::dsp::DspSettings;

pub struct AudioBridge {
    settings: Arc<RwLock<DspSettings>>,
    running: Arc<AtomicBool>,
    selected_input: Arc<RwLock<Option<String>>>,
    selected_output: Arc<RwLock<Option<String>>>,
    reload_requested: Arc<AtomicBool>,
    visualizer: Arc<VisualizerData>,
}

impl AudioBridge {
    pub fn new(
        settings: Arc<RwLock<DspSettings>>,
        selected_input: Arc<RwLock<Option<String>>>,
        selected_output: Arc<RwLock<Option<String>>>,
        reload_requested: Arc<AtomicBool>,
        visualizer: Arc<VisualizerData>,
    ) -> Self {
        Self {
            settings,
            running: Arc::new(AtomicBool::new(true)),
            selected_input,
            selected_output,
            reload_requested,
            visualizer,
        }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn run(&self) {
        #[cfg(windows)]
        unsafe {
            let _ = windows::Win32::System::Com::CoInitializeEx(
                None,
                windows::Win32::System::Com::COINIT_MULTITHREADED,
            );
        }

        let host = cpal::default_host();

        let mut rdp_warned = false;

        while self.running.load(Ordering::SeqCst) {
            self.reload_requested.store(false, Ordering::SeqCst);

            let devices = match host.output_devices() {
                Ok(devs) => devs.collect::<Vec<_>>(),
                Err(e) => {
                    eprintln!(">>> Aygıtlar listelenemedi: {}. Bekleniyor...", e);
                    std::thread::sleep(Duration::from_millis(1500));
                    continue;
                }
            };

            if devices.is_empty() {
                eprintln!(">>> [Bekleniyor] Hiçbir ses aygıtı bulunamadı. Yeniden taranıyor...");
                std::thread::sleep(Duration::from_millis(1500));
                continue;
            }

            // 1. ÇIKIŞ AYGITINI BELİRLE (Fiziksel Hoparlör / Monitör / RDP)
            let out_pref = self.selected_output.read().clone();
            let output_device = if let Some(ref target) = out_pref {
                devices.iter().find(|d| {
                    d.description().map(|desc| desc.name() == target).unwrap_or(false)
                }).cloned().unwrap_or_else(|| self.auto_find_real_output(&devices))
            } else {
                self.auto_find_real_output(&devices)
            };

            let out_name = output_device.description()
                .map(|d| d.name().to_string())
                .unwrap_or_else(|_| "Bilinmeyen Çıkış".to_string());
            *self.selected_output.write() = Some(out_name.clone());

            // Fiziksel çıkış aygıtının ses düzeyini otomatik olarak %100'e getir (Kısık kalmayı engelle)
            crate::engine::endpoint_volume::maximize_endpoint_volume(&out_name);

            // 2. GİRİŞ AYGITINI BELİRLE (Sanal Aygıt / Loopback Kaynağı)
            let in_pref = self.selected_input.read().clone();
            let capture_device = if let Some(ref target) = in_pref {
                if target == &out_name {
                    self.auto_find_virtual_sink(&devices, &out_name)
                } else {
                    devices.iter().find(|d| {
                        d.description().map(|desc| desc.name() == target).unwrap_or(false)
                    }).cloned().unwrap_or_else(|| self.auto_find_virtual_sink(&devices, &out_name))
                }
            } else {
                self.auto_find_virtual_sink(&devices, &out_name)
            };

            let cap_name = capture_device.description()
                .map(|d| d.name().to_string())
                .unwrap_or_else(|_| "Bilinmeyen Giriş".to_string());
            *self.selected_input.write() = Some(cap_name.clone());

            if cap_name == out_name {
                if !rdp_warned {
                    if out_name.contains("Remote Audio") {
                        println!("\n============================================================");
                        println!(">>> [RDP OTURUMU ALGILANDI: Yalnızca 'Remote Audio' mevcut]");
                        println!(">>> Windows RDP ses yönlendirmesi ('Bu bilgisayarda çal') aktifken");
                        println!(">>> Windows, yerel ve sanal aygıtları ('Posea Audio' vb.) bu oturumdan gizler.");
                        println!(">>> ");
                        println!(">>> Çözüm Seçenekleri:");
                        println!(">>> 1) RDP İstemcisinde (mstsc):");
                        println!(">>>    Seçenekler -> Yerel Kaynaklar -> Uzak Ses Ayarları");
                        println!(">>>    'Uzak bilgisayarda çal' (Play on remote computer) seçilirse,");
                        println!(">>>    'Posea Audio' ve hoparlörler doğrudan görünür.");
                        println!(">>> 2) 'mstsc /admin' komutu ile konsol oturumuna bağlanırsanız,");
                        println!(">>>    tüm ses donanımları doğrudan erişilebilir olur.");
                        println!(">>> 3) Yerel masaüstüne dönüldüğünde Posea Audio otomatik bağlanır.");
                        println!("============================================================\n");
                    } else {
                        eprintln!(">>> [UYARI] Giriş ve Çıkış aynı aygıt ({})! Geri bildirim döngüsünü engellemek için bekleniyor...", cap_name);
                    }
                    rdp_warned = true;
                }
                std::thread::sleep(Duration::from_millis(3000));
                continue;
            }
            rdp_warned = false;

            println!("\n============================================================");
            println!(">>> [AKTİF GİRİŞ (Sanal)]: [{}] <- Windows/YouTube", cap_name);
            println!(">>> [AKTİF ÇIKIŞ (Hoparlör)]: [{}] <- Güçlendirilmiş Ses", out_name);
            println!("============================================================");

            // 3. Konfigürasyonlar
            let out_config = match output_device.default_output_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!(">>> Çıkış formatı alınamadı ({}). Yeniden deneniyor...", e);
                    std::thread::sleep(Duration::from_millis(1000));
                    continue;
                }
            };

            let out_sample_rate = out_config.sample_rate() as f32;
            let out_channels = out_config.channels() as usize;

            let cap_config = match capture_device.default_output_config() {
                Ok(c) => c,
                Err(_) => out_config.clone(),
            };

            let cap_channels = cap_config.channels() as usize;

            // 4. Ring Buffer (16384 float)
            let ring_buffer = HeapRb::<f32>::new(16384);
            let (mut producer, mut consumer) = ring_buffer.split();

            let device_lost = Arc::new(AtomicBool::new(false));
            let dev_lost_in = device_lost.clone();
            let dev_lost_out = device_lost.clone();

            // 5. Giriş Akışı (Loopback Capture)
            let cap_stream_cfg: cpal::StreamConfig = cap_config.into();
            let input_stream_res = capture_device.build_input_stream(
                cap_stream_cfg,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    for &sample in data {
                        let _ = producer.try_push(sample);
                    }
                },
                move |err| {
                    let err_str = err.to_string();
                    if err_str.contains("underrun") || err_str.contains("overrun") {
                        return;
                    }
                    eprintln!("\n>>> [Giriş Akış Uyarısı]: {}", err);
                    dev_lost_in.store(true, Ordering::SeqCst);
                },
                None,
            );

            let input_stream = match input_stream_res {
                Ok(s) => s,
                Err(e) => {
                    eprintln!(">>> Giriş akışı başlatılamadı: {}. Bekleniyor...", e);
                    std::thread::sleep(Duration::from_millis(1500));
                    continue;
                }
            };

            // 6. Çıkış Akışı (DSP -> Hoparlör)
            let mut dsp = DspProcessor::new(out_sample_rate);
            let mut limiter = FastLimiter::new(out_sample_rate);
            let mut analyzer = VisualizerAnalyzer::new(self.visualizer.clone());
            let settings_ref = self.settings.clone();
            let mut last_version = 0u64;

            let out_stream_cfg: cpal::StreamConfig = out_config.into();
            let output_stream_res = output_device.build_output_stream(
                out_stream_cfg,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let active_settings = settings_ref.read().clone();
                    if active_settings.version != last_version {
                        dsp.update_settings(&active_settings);
                        last_version = active_settings.version;
                    }

                    for frame in data.chunks_mut(out_channels) {
                        let in_l = consumer.try_pop().unwrap_or(0.0);
                        let in_r = if cap_channels > 1 {
                            consumer.try_pop().unwrap_or(in_l)
                        } else {
                            in_l
                        };

                        let (l, r) = dsp.process_stereo(in_l, in_r, &active_settings);
                        let (out_l, out_r) = limiter.process_stereo(l, r);

                        // Spektrum ve tepe seviye görselleştiriciyi besle
                        analyzer.add_sample(out_l, out_r);

                        if out_channels >= 2 {
                            frame[0] = out_l;
                            frame[1] = out_r;
                        } else if out_channels == 1 {
                            frame[0] = 0.5 * (out_l + out_r);
                        }
                    }
                },
                move |err| {
                    let err_str = err.to_string();
                    if err_str.contains("underrun") || err_str.contains("overrun") {
                        return;
                    }
                    eprintln!("\n>>> [Çıkış Akış Uyarısı]: {}", err);
                    dev_lost_out.store(true, Ordering::SeqCst);
                },
                None,
            );

            let output_stream = match output_stream_res {
                Ok(s) => s,
                Err(e) => {
                    eprintln!(">>> Çıkış akışı başlatılamadı: {}. Bekleniyor...", e);
                    std::thread::sleep(Duration::from_millis(1500));
                    continue;
                }
            };

            let _ = input_stream.play();
            let _ = output_stream.play();

            println!(">>> [KÖPRÜ AKTİF ÇALIŞIYOR!]");
            println!("    Windows ses çıkışını '{}' yapın,", cap_name);
            println!("    Ses '{}' hoparlörünüzden güçlendirilmiş olarak gelecektir.\n", out_name);

            while !device_lost.load(Ordering::SeqCst) 
                && !self.reload_requested.load(Ordering::SeqCst) 
                && self.running.load(Ordering::SeqCst) 
            {
                std::thread::sleep(Duration::from_millis(200));
            }

            drop(input_stream);
            drop(output_stream);
            std::thread::sleep(Duration::from_millis(300));
        }
    }

    fn auto_find_real_output(&self, devices: &[cpal::Device]) -> cpal::Device {
        // 1. RDP oturumu varsa Remote Audio ilk tercih
        for dev in devices {
            if let Ok(desc) = dev.description() {
                if desc.name().contains("Remote Audio") {
                    return dev.clone();
                }
            }
        }
        // 2. Fiziksel hoparlör/monitör/kulaklık seç (Posea ve sanal aygıtlar hariç)
        for dev in devices {
            if let Ok(desc) = dev.description() {
                let name = desc.name();
                if !name.contains("Posea") 
                    && !name.contains("Virtual") 
                    && !name.contains("Sonic Studio") 
                    && !name.to_lowercase().contains("virtual") 
                    && !name.contains("Digital Output") 
                {
                    if name.contains("LG") || name.contains("AMD") || name.contains("Realtek") || name.contains("Hoparlör") {
                        return dev.clone();
                    }
                }
            }
        }
        // 3. Posea olmayan herhangi bir aygıt
        for dev in devices {
            if let Ok(desc) = dev.description() {
                let name = desc.name();
                if !name.contains("Posea") && !name.contains("Virtual") {
                    return dev.clone();
                }
            }
        }
        devices[0].clone()
    }

    fn auto_find_virtual_sink(&self, devices: &[cpal::Device], real_out: &str) -> cpal::Device {
        // 1. Öncelik: Resmi Posea Audio Sanal Sürücüsü
        for dev in devices {
            if let Ok(desc) = dev.description() {
                let name = desc.name();
                if name != real_out && name.contains("Posea") {
                    return dev.clone();
                }
            }
        }
        // 2. Yedek: Diğer sanal aygıtlar
        for dev in devices {
            if let Ok(desc) = dev.description() {
                let name = desc.name();
                if name != real_out && (name.contains("Virtual") || name.contains("Mixer")) {
                    return dev.clone();
                }
            }
        }
        devices.iter().find(|d| {
            d.description().map(|desc| desc.name() != real_out).unwrap_or(false)
        }).cloned().unwrap_or_else(|| devices[0].clone())
    }
}
