@echo off
setlocal
cd /d "%~dp0"

echo ============================================================
echo   PoseaFXMixer - Posea Audio Sanal Ses Surucusu Kurulumu
echo ============================================================

net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [BILGI] Yonetici yetkisi isteniyor...
    powershell -Command "Start-Process cmd -ArgumentList '/k \"\"%~f0\"\"' -Verb RunAs"
    exit /b
)

echo [1/4] Posea Root Sertifikasi Guvenilen Depolara Ekleniyor...
if exist PoseaCert.cer (
    certutil -addstore "Root" PoseaCert.cer >nul 2>&1
    certutil -addstore "TrustedPublisher" PoseaCert.cer >nul 2>&1
    echo [OK] Sertifika sisteme yuklendi.
) else (
    echo [UYARI] PoseaCert.cer bulunamadi!
)

echo.
echo [2/4] Eski Posea surucu kayitlari temizleniyor...
devcon.exe remove *PoseaAudio >nul 2>&1

echo.
echo [3/4] Sürücü paketi Windows Surucu Deposuna ekleniyor...
pnputil /add-driver PoseaAudio.inf /install

echo.
echo [4/4] Posea Audio Sanal Aygiti olusturuluyor...
devcon.exe install PoseaAudio.inf *PoseaAudio

echo.
echo ============================================================
echo [TEBRIKLER] 'Posea Audio Virtual Device' kurulum komutlari tamamlandi!
echo Windows Ses aygitlarinda ve Cihaz Yoneticisinde kontrol edebilirsiniz.
echo ============================================================
pause
