@echo off
setlocal
cd /d "%~dp0"

echo =======================================================================
echo              PoseaFXMixer - Windows Yeniden Baslatma ve Surucu Yonetimi
echo =======================================================================
echo.
echo Windows x64 suruculerini (Posea Audio) yuklemek icin iki yontem vardir:
echo.
echo [1] GELISMIS BASLANGIC ILE YENIDEN BASLAT (Surucu Imzasi Kapatma - F7)
echo     Windows mavi ekranda acilir:
echo     Sorun Gider -> Gelismis secenekler -> Baslangic Ayarlari -> Yeniden Baslat
echo     Acilis ekraninda klavyeden 7 veya F7'ye basin.
echo.
echo [2] TEST MODUNDA YENIDEN BASLAT (TestSigning On)
echo     Test modu bcdedit ile onceden acilmistir. 
echo     Normal yeniden baslatma sonrasi sag altta 'Test Modu' yazar.
echo.
echo [3] POSEA AUDIO SURUCUSUNU SIMDI KUR (install_posea_driver.cmd)
echo     Yeniden baslatma yaptiktan sonra bu secenegi secin veya
echo     drivers\windows\install_posea_driver.cmd dosyasini calistirin.
echo.
echo [4] CIKIS
echo.
echo =======================================================================
set /p choice="Seciminizi yapin [1, 2, 3, 4]: "

if "%choice%"=="1" goto advanced_reboot
if "%choice%"=="2" goto normal_reboot
if "%choice%"=="3" goto install_driver
if "%choice%"=="4" goto exit_script

echo Gecersiz secim!
pause
exit /b

:advanced_reboot
echo.
echo [BILGI] Gelismis Baslangic Moduna geciliyor...
echo Bilgisayariniz simdi mavi ekranda yeniden baslatilacak!
echo Acilinca sirasiyla:
echo 1. Sorun Gider (Troubleshoot)
echo 2. Gelismis secenekler (Advanced options)
echo 3. Baslangic Ayarlari (Startup Settings)
echo 4. Yeniden Baslat (Restart)
echo 5. Listede '7' veya 'F7' tusuna basin ('Surucu imzasi zorlamasini devre disi birak')
echo.
pause
shutdown.exe /r /o /f /t 00
exit /b

:normal_reboot
echo.
echo [BILGI] Test modunun aktif olmasi icin bilgisayar yeniden baslatiliyor...
echo.
pause
shutdown.exe /r /f /t 00
exit /b

:install_driver
call "%~dp0drivers\windows\install_posea_driver.cmd"
exit /b

:exit_script
exit /b
