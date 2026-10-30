@echo off
setlocal
cd /d "%~dp0"

echo =======================================================================
echo    Posea Audio - Gelismis Baslangic (Surucu Imzasi Kapatma) ile Yeniden Baslat
echo =======================================================================
echo.
echo Bu islem bilgisayarinizi 'Gelismis Baslangic' menusunde yeniden baslatacaktir.
echo.
echo Mavi ekran acildiginda su adimlari takip edin:
echo  1. "Sorun Gider" (Troubleshoot) secin
echo  2. "Gelismis secenekler" (Advanced options) secin
echo  3. "Baslangic Ayarlari" (Startup Settings) secin
echo  4. "Yeniden Baslat" (Restart) butonuna basin
echo  5. Ekrana gelen numarali listede "7" veya "F7" tusuna basin:
echo     ("Surucu imzasi zorlamasini devre disi birak" / "Disable driver signature enforcement")
echo.
echo Windows acildiktan sonra bu klasordeki "install_posea_driver.cmd" dosyasini calistirin!
echo =======================================================================
echo.
pause
shutdown.exe /r /o /f /t 00
