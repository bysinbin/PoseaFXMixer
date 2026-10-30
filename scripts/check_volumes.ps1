$src = @'
using System;
using System.Runtime.InteropServices;

namespace AudioVol {
    [Guid("5CDF2C82-841E-4546-9722-0CF74078229A"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    public interface IAudioEndpointVolume {
        int SetMasterVolumeLevel(float fLevelDB, ref Guid pguidEventContext);
        int SetMasterVolumeLevelScalar(float fLevel, ref Guid pguidEventContext);
        int GetMasterVolumeLevel(out float pfLevelDB);
        int GetMasterVolumeLevelScalar(out float pfLevel);
        int SetMute(bool bMute, ref Guid pguidEventContext);
        int GetMute(out bool pbMute);
    }

    [Guid("D666063F-1587-4E43-81F1-B948E807363F"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    public interface IMMDevice {
        int Activate(ref Guid id, int clsCtx, IntPtr activationParams, [MarshalAs(UnmanagedType.IUnknown)] out object val);
    }

    [Guid("A95664D2-9614-4F35-A746-DE8DB63617E6"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    public interface IMMDeviceEnumerator {
        int EnumAudioEndpoints(int dataFlow, int dwStateMask, out IntPtr ppDevices);
        int GetDefaultAudioEndpoint(int dataFlow, int role, out IMMDevice ppEndpoint);
    }

    [ComImport, Guid("BCDE0395-E52F-467C-8E3D-C4579291692E")]
    public class MMDeviceEnumeratorComObject { }

    public class VolHelper {
        public static float GetDefaultVol() {
            var enumerator = (IMMDeviceEnumerator)new MMDeviceEnumeratorComObject();
            IMMDevice dev;
            int hr = enumerator.GetDefaultAudioEndpoint(0, 1, out dev);
            if (hr != 0 || dev == null) return -1f;
            Guid iid = new Guid("5CDF2C82-841E-4546-9722-0CF74078229A");
            object epvObj;
            dev.Activate(ref iid, 1, IntPtr.Zero, out epvObj);
            var epv = (IAudioEndpointVolume)epvObj;
            float level;
            epv.GetMasterVolumeLevelScalar(out level);
            return level;
        }

        public static void SetDefaultVol(float vol) {
            var enumerator = (IMMDeviceEnumerator)new MMDeviceEnumeratorComObject();
            IMMDevice dev;
            int hr = enumerator.GetDefaultAudioEndpoint(0, 1, out dev);
            if (hr != 0 || dev == null) return;
            Guid iid = new Guid("5CDF2C82-841E-4546-9722-0CF74078229A");
            object epvObj;
            dev.Activate(ref iid, 1, IntPtr.Zero, out epvObj);
            var epv = (IAudioEndpointVolume)epvObj;
            Guid ctx = Guid.Empty;
            epv.SetMasterVolumeLevelScalar(vol, ref ctx);
        }
    }
}
'@

Add-Type -TypeDefinition $src
$vol = [AudioVol.VolHelper]::GetDefaultVol()
Write-Host "Windows Varsayılan Ses Düzeyi: $([math]::Round($vol * 100))%"
