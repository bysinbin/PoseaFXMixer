# Posea Audio - Sertifika Ekleme ve Imzalama Dogrulama Betigi
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "[BILGI] Yonetici haklariyla yeniden baslatiliyor..." -ForegroundColor Yellow
    Start-Process powershell.exe -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`"" -Verb RunAs
    exit
}

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "   Posea Audio - Gelistirici Sertifikasi Yukleme" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

# 1. PoseaCert.cer dosyasını Güvenilen Kök Yetkililerine ve Yayıncılara Ekle
$cerPath = Join-Path $ScriptDir "PoseaCert.cer"
if (Test-Path $cerPath) {
    Write-Host "Posea Kök Sertifikası Windows'a güvenilir olarak ekleniyor..." -ForegroundColor Yellow
    
    # Root (Kök)
    $rootStore = New-Object System.Security.Cryptography.X509Certificates.X509Store "Root", "LocalMachine"
    $rootStore.Open("ReadWrite")
    $certObj = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2 $cerPath
    $rootStore.Add($certObj)
    $rootStore.Close()

    # TrustedPublisher (Güvenilen Yayıncı)
    $pubStore = New-Object System.Security.Cryptography.X509Certificates.X509Store "TrustedPublisher", "LocalMachine"
    $pubStore.Open("ReadWrite")
    $pubStore.Add($certObj)
    $pubStore.Close()

    Write-Host "[BASARILI] Posea Technologies Root CA sisteme eklendi!" -ForegroundColor Green
} else {
    Write-Host "[HATA] PoseaCert.cer bulunamadi!" -ForegroundColor Red
}

# 2. Imza Dogrulama Kontrolu
$signtool = (Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin\*\x64\signtool.exe" | Select-Object -Last 1).FullName
$sysPath = Join-Path $ScriptDir "PoseaAudio.sys"
Write-Host "`nPoseaAudio.sys imza durumu doğrulanıyor..." -ForegroundColor Yellow
& $signtool verify /v /pa $sysPath

Write-Host "`n============================================================" -ForegroundColor Green
Write-Host "Sertifika islemi tamamlandi!" -ForegroundColor Green
Write-Host "Simdi bilgisayari yeniden baslatip 'install_posea_driver.cmd' calistirabilirsiniz." -ForegroundColor White
Write-Host "============================================================" -ForegroundColor Green
Read-Host "Devam etmek icin Enter'a basin..."
