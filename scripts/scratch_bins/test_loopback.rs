use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Producer, Split};
use ringbuf::HeapRb;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_MULTITHREADED,
        );
    }

    println!("============================================================");
    println!("     PoseaFXMixer - RDP Ses Akışı ve Loopback Testi         ");
    println!("============================================================");

    let host = cpal::default_host();
    println!("Ses Hostu: {:?}", host.id());

    // 1. Tüm Aygıtları Tara
    println!("\n--- Host Çıkış Aygıtları ---");
    let mut all_devices = Vec::new();
    if let Ok(devs) = host.output_devices() {
        for (i, dev) in devs.enumerate() {
            let name = dev.description().map(|d| d.name().to_string()).unwrap_or_default();
            let cfg_str = dev.default_output_config()
                .map(|c| format!("Kanallar: {}, Hz: {}, Format: {:?}", c.channels(), c.sample_rate(), c.sample_format()))
                .unwrap_or_else(|e| format!("Hata: {}", e));
            println!("  [{}] {} ({})", i + 1, name, cfg_str);
            all_devices.push((name, dev));
        }
    } else {
        println!("Çıkış aygıtları listelenemedi!");
    }

    println!("\n--- Host Giriş Aygıtları ---");
    if let Ok(devs) = host.input_devices() {
        for (i, dev) in devs.enumerate() {
            let name = dev.description().map(|d| d.name().to_string()).unwrap_or_default();
            println!("  [{}] {}", i + 1, name);
        }
    }

    let default_output = host.default_output_device();
    println!("\nVarsayılan Çıkış: {:?}", default_output.as_ref().map(|d| d.description().map(|desc| desc.name().to_string())));

    let out_dev = match default_output {
        Some(d) => d,
        None => {
            println!("HATA: Varsayılan çıkış aygıtı yok!");
            return Ok(());
        }
    };

    let out_desc = out_dev.description()?;
    println!("Seçilen Çıkış: {}", out_desc.name());

    let out_cfg = out_dev.default_output_config()?;
    println!("Çıkış Formatı: {:?}, Kanallar: {}, Hz: {}", out_cfg.sample_format(), out_cfg.channels(), out_cfg.sample_rate());

    // 2. Loopback Yakalama Testi: Çıkış aygıtından build_input_stream çağrılabiliyor mu?
    println!("\n--- WASAPI Loopback Yakalama Denemesi ---");
    let stream_cfg: cpal::StreamConfig = out_cfg.into();

    let ring_buffer = HeapRb::<f32>::new(8192);
    let (mut prod, mut cons) = ring_buffer.split();

    let input_res = out_dev.build_input_stream(
        stream_cfg.clone(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            // Sinyal gücünü ölç
            let mut max_val = 0.0f32;
            for &s in data {
                max_val = max_val.max(s.abs());
                let _ = prod.try_push(s);
            }
            if max_val > 0.01 {
                println!(">> [SES YAKALANDI!] Anlık Tepe Seviyesi: {:.4}", max_val);
            }
        },
        |err| eprintln!("Giriş akış hatası: {}", err),
        None,
    );

    match input_res {
        Ok(stream) => {
            println!("BAŞARILI: out_dev üzerinde Loopback input stream açıldı!");
            match stream.play() {
                Ok(_) => println!("BAŞARILI: Loopback akışı dinlemeye başladı! (5 saniye dinleniyor, lütfen YouTube'dan bir video oynatın...)"),
                Err(e) => println!("HATA: Loopback stream.play() başarısız: {}", e),
            }
            std::thread::sleep(Duration::from_secs(5));
        }
        Err(e) => {
            println!("BAŞARISIZ: Loopback input stream açılamadı! Hata: {}", e);
        }
    }

    Ok(())
}
