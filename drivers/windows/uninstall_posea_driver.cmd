@echo off
setlocal
cd /d "%~dp0"

echo ============================================================
echo   PoseaFXMixer - Posea Audio Sanal Ses Surucusu Kaldirma
echo ============================================================

net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [BILGI] Yonetici yetkisi isteniyor...
    powershell -Command "Start-Process cmd -ArgumentList '/k \"\"%~f0\"\"' -Verb RunAs"
    exit /b
)

echo Posea Audio surucusu sistemden kaldiriliyor...
devcon.exe remove *PoseaAudio

echo.
echo [TAMAMLANDI] Posea Audio surucusu kaldirildi.
pause
