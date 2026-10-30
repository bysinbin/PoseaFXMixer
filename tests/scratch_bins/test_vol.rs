use windows::core::*;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;

fn main() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let collection = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;
        let count = collection.GetCount()?;

        for i in 0..count {
            let device = collection.Item(i)?;
            let store = device.OpenPropertyStore(STGM_READ)?;
            // Property key: PKEY_Device_FriendlyName ({a45c254e-df1c-4efd-8020-67d146a850e0}, 2)
            let prop = store.GetValue(&windows::Win32::Foundation::PROPERTYKEY {
                fmtid: windows::core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
                pid: 2,
            })?;

            let name = prop.Anonymous.Anonymous.Anonymous.pwszVal.to_string().unwrap_or_default();
            println!("Aygıt [{}]: {}", i, name);

            if name.contains("LG") || name.contains("AMD") {
                let epv: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
                let current_vol = epv.GetMasterVolumeLevelScalar()?;
                println!("  -> Eski Ses Düzeyi: {:.0}%", current_vol * 100.0);
                epv.SetMasterVolumeLevelScalar(1.0, std::ptr::null())?;
                epv.SetMute(false, std::ptr::null())?;
                println!("  -> [GÜNCELLENDİ] Ses %100 yapıldı ve Sessizden çıkarıldı!");
            }
        }
    }
    Ok(())
}
