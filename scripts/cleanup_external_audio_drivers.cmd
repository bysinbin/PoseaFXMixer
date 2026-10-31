@echo off
setlocal
cd /d "%~dp0"

echo ======================================================================
echo   FxSound ve Sonic Studio Suruculerini Sistemden Kaldirma Scripti
echo ======================================================================
echo.

net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [BILGI] Yonetici haklari isteniyor...
    powershell -Command "Start-Process cmd -ArgumentList '/c \"\"%~f0\"\"' -Verb RunAs"
    exit /b
)

echo [1/4] FxSound Audio Enhancer sanal aygiti kaldiriliyor...
pnputil /remove-device "ROOT\MEDIA\0000" >nul 2>&1
pnputil /delete-driver oem26.inf /uninstall /force >nul 2>&1

echo [2/4] Sonic Studio Virtual Mixer sanal aygiti kaldiriliyor...
pnputil /remove-device "SWD\DRIVERENUM\{1b98ef90-ca0e-11e7-8f1a-0800200c9a66}#AVoluteSS3Vad&6&25ad6b79&0" >nul 2>&1
pnputil /delete-driver oem107.inf /uninstall /force >nul 2>&1

echo [3/4] Ek olarak kayitli A-Volute Nahimic veya FxSound nesneleri kontrol ediliyor...
for /f "tokens=2 delims=:" %%a in ('pnputil /enum-devices /class Media ^| findstr /i "FxSound"') do (
    pnputil /remove-device "%%a" >nul 2>&1
)

echo [4/4] Windows Ses Aygit Listesi Yenileniyor...
powershell -Command "Restart-Service Audiosrv -Force" >nul 2>&1

echo.
echo ======================================================================
echo   [TAMAMLANDI] FxSound ve Sonic Studio suruculeri basariyla kaldirildi!
echo ======================================================================
timeout /t 5
