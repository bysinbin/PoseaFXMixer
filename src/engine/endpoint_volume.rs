/// Windows Ses Uç Noktası (Endpoint) Donanım Ses Düzeyi Yöneticisi
/// Sanal mikser çalışırken arka plandaki fiziksel aygıtın (Örn: LG Monitör)
/// sesinin kısık kalmasını engellemek için sesini otomatik olarak %100'e çeker ve sessizden çıkarır.

#[cfg(target_os = "windows")]
pub fn maximize_endpoint_volume(target_name: &str) {
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_MULTITHREADED,
        );

        use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
        use windows::Win32::Media::Audio::*;
        use windows::Win32::System::Com::*;

        let enumerator: IMMDeviceEnumerator = match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
            Ok(e) => e,
            Err(_) => return,
        };

        let collection = match enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE) {
            Ok(c) => c,
            Err(_) => return,
        };

        let count = match collection.GetCount() {
            Ok(c) => c,
            Err(_) => return,
        };

        for i in 0..count {
            if let Ok(device) = collection.Item(i) {
                if let Ok(store) = device.OpenPropertyStore(STGM_READ) {
                    let key = windows::Win32::Foundation::PROPERTYKEY {
                        fmtid: windows::core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
                        pid: 2,
                    };

                    if let Ok(prop) = store.GetValue(&key) {
                        let name = prop.Anonymous.Anonymous.Anonymous.pwszVal.to_string().unwrap_or_default();
                        // Hedef aygıt adıyla eşleşiyorsa (Posea Audio hariç)
                        if !name.is_empty() 
                            && !name.contains("Posea") 
                            && (name.contains(target_name) || target_name.contains(&name)) 
                        {
                            if let Ok(epv) = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None) {
                                let prev_vol = epv.GetMasterVolumeLevelScalar().unwrap_or(1.0);
                                let _ = epv.SetMasterVolumeLevelScalar(1.0, std::ptr::null());
                                let _ = epv.SetMute(false, std::ptr::null());
                                println!(">>> [OTOMATİK DONANIM SESİ] '{}' ses düzeyi %{:.0}'den %100'e getirildi (Tam Güç).", name, prev_vol * 100.0);
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn maximize_endpoint_volume(_target_name: &str) {}
