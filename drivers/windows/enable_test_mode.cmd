@echo off
setlocal
cd /d "%~dp0"

echo ============================================================
echo   PoseaFXMixer - Windows Test Modunu Acma
echo ============================================================

net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [BILGI] Yonetici yetkisi isteniyor...
    powershell -Command "Start-Process cmd -ArgumentList '/k \"\"%~f0\"\"' -Verb RunAs"
    exit /b
)

echo.
echo Windows Surucu Test Modu etkinlestiriliyor...
bcdedit /set testsigning on

echo.
echo ============================================================
echo [BASARILI] TestSigning modu basariyla acildi!
echo.
echo Lutfen sirasiyla:
echo 1. 'create_cert_and_sign.ps1' calistirarak Posea sertifikasini imzalayin.
echo 2. Bilgisayarinizi YENIDEN BASLATIN.
echo 3. Yeniden baslayinca 'install_posea_driver.cmd' calistirin.
echo ============================================================
pause
