use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use posea_fx_mixer::dsp::effects::DspProcessor;
use posea_fx_mixer::dsp::limiter::FastLimiter;
use posea_fx_mixer::dsp::DspSettings;

fn main() -> anyhow::Result<()> {
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_MULTITHREADED,
        );
    }

    println!("============================================================");
    println!("     PoseaFXMixer - Canlı Ses Akışı ve DSP Test Aracı       ");
    println!("============================================================");

    let host = cpal::default_host();
    let devices: Vec<_> = host.output_devices()?.collect();

    println!("\nBulunan Çıkış Aygıtları:");
    for (i, dev) in devices.iter().enumerate() {
        let name = dev.description().map(|d| d.name().to_string()).unwrap_or_default();
        println!("  [{}] {}", i + 1, name);
    }

    // Çıkış (Fiziksel Monitör / LG HDR WQHD)
    let output_device = devices.iter().find(|d| {
        let name = d.description().map(|desc| desc.name().to_string()).unwrap_or_default();
        name.contains("LG") || name.contains("AMD") || name.contains("Realtek")
    }).expect("Fiziksel çıkış aygıtı (LG/Realtek) bulunamadı!").clone();

    // Giriş (Sanal Aygıt / Sonic Studio)
    let capture_device = devices.iter().find(|d| {
        let name = d.description().map(|desc| desc.name().to_string()).unwrap_or_default();
        name.contains("Sonic Studio") || name.contains("Virtual") || name.contains("FxSound")
    }).expect("Sanal giriş aygıtı (Sonic Studio/FxSound) bulunamadı!").clone();

    let out_name = output_device.description()?.name().to_string();
    let cap_name = capture_device.description()?.name().to_string();

    println!("\n>>> GİRİŞ KAYNAĞI (Windows Sesini Buraya Verin): [{}]", cap_name);
    println!(">>> NİHAİ DİNLEME ÇIKIŞI (Hoparlörünüz):          [{}]\n", out_name);

    let out_cfg = output_device.default_output_config()?;
    let out_sr = out_cfg.sample_rate() as f32;
    let out_channels = out_cfg.channels() as usize;

    let ring_buffer = HeapRb::<f32>::new(8192);
    let (mut producer, mut consumer) = ring_buffer.split();

    let running = Arc::new(AtomicBool::new(true));
    let r1 = running.clone();
    let r2 = running.clone();

    // DSP Ayarları (Belirgin fark için güçlü Bas ve Netlik)
    let mut settings = DspSettings::default();
    settings.bass_boost = 7.0;
    settings.clarity = 5.0;
    settings.surround = 6.0;
    let mut dsp = DspProcessor::new(out_sr);
    let mut limiter = FastLimiter::new(out_sr);
    dsp.update_settings(&settings);

    // 1. Giriş Akışı (Sonic Studio üzerinden Loopback)
    let in_cfg = capture_device.default_output_config()?;
    let in_channels = in_cfg.channels() as usize;

    let in_stream = capture_device.build_input_stream(
        in_cfg.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut peak = 0.0f32;
            for &s in data {
                peak = peak.max(s.abs());
                let _ = producer.try_push(s);
            }
            if peak > 0.02 {
                print!("\r>> [SES GİRİŞİ ALINDI!] Seviye: {:.3} | DSP İşleniyor...", peak);
            }
        },
        move |err| {
            eprintln!("\nGiriş hatası: {}", err);
            r1.store(false, Ordering::SeqCst);
        },
        None,
    )?;

    // 2. Çıkış Akışı (LG HDR WQHD)
    let out_stream = output_device.build_output_stream(
        out_cfg.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(out_channels) {
                let in_l = consumer.try_pop().unwrap_or(0.0);
                let in_r = if in_channels > 1 {
                    consumer.try_pop().unwrap_or(in_l)
                } else {
                    in_l
                };

                let (l, r) = dsp.process_stereo(in_l, in_r, &settings);
                let (out_l, out_r) = limiter.process_stereo(l, r);

                if out_channels >= 2 {
                    frame[0] = out_l;
                    frame[1] = out_r;
                } else if out_channels == 1 {
                    frame[0] = 0.5 * (out_l + out_r);
                }
            }
        },
        move |err| {
            eprintln!("\nÇıkış hatası: {}", err);
            r2.store(false, Ordering::SeqCst);
        },
        None,
    )?;

    in_stream.play()?;
    out_stream.play()?;

    println!(">>> [KÖPRÜ DEVREDE]");
    println!("Lütfen şimdi Windows Ses simgesinden çıkışı '{}' yapın ve bir şarkı/video açın.", cap_name);
    println!("Ses doğrudan '{}' monitörünüze güçlendirilmiş olarak gelecektir!", out_name);
    println!("(Durdurmak için 15 saniye sonra otomatik kapanacak veya izleyebilirsiniz)\n");

    for _ in 0..15 {
        if !running.load(Ordering::SeqCst) {
            break;
        }
        std::thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}
