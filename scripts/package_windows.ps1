# Posea Audio Windows Packaging Script
param(
    [string]$Version = "1.0.0"
)

$ErrorActionPreference = "Stop"

Write-Host ">>> [1/3] Building Posea Audio in Release Mode..." -ForegroundColor Cyan
cargo build --release

$distDir = "dist"
if (Test-Path $distDir) {
    Remove-Item -Recurse -Force $distDir
}
New-Item -ItemType Directory -Force -Path "$distDir\windows" | Out-Null

$releaseExe = "target\release\posea_fx_mixer.exe"
if (-not (Test-Path $releaseExe)) {
    Write-Error "Release binary not found at $releaseExe"
}

Write-Host ">>> [2/3] Creating Portable Windows ZIP Bundle..." -ForegroundColor Cyan
$portableFolder = "$distDir\windows\PoseaAudio-v$Version-Windows-x64"
New-Item -ItemType Directory -Force -Path $portableFolder | Out-Null

Copy-Item $releaseExe -Destination "$portableFolder\PoseaAudio.exe"
Copy-Item "assets\icon.ico" -Destination "$portableFolder\"
Copy-Item "assets\icon.png" -Destination "$portableFolder\"
if (Test-Path "drivers") {
    Copy-Item -Recurse "drivers" -Destination "$portableFolder\drivers"
}
Copy-Item "README.md" -Destination "$portableFolder\" -ErrorAction SilentlyContinue

$zipPath = "$distDir\PoseaAudio-v$Version-Windows-x64-Portable.zip"
Compress-Archive -Path "$portableFolder\*" -DestinationPath $zipPath -Force
Write-Host "    Portable ZIP created: $zipPath" -ForegroundColor Green

Write-Host ">>> [3/3] Checking WiX Toolset for MSI Installer..." -ForegroundColor Cyan
if ((Get-Command "candle.exe" -ErrorAction SilentlyContinue) -and (Get-Command "light.exe" -ErrorAction SilentlyContinue)) {
    Write-Host "    Compiling WiX MSI installer..." -ForegroundColor Yellow
    candle.exe -nologo -out "$distDir\windows\posea.wixobj" "packaging\windows\posea_audio.wxs"
    light.exe -nologo -ext WixUIExtension -out "$distDir\PoseaAudio-v$Version-Windows-x64.msi" "$distDir\windows\posea.wixobj"
    Write-Host "    MSI Installer created: $distDir\PoseaAudio-v$Version-Windows-x64.msi" -ForegroundColor Green
} else {
    Write-Host "    WiX Toolset (candle/light) not found on local PATH. MSI compilation will be executed automatically on GitHub Actions CI." -ForegroundColor Yellow
}

Write-Host ">>> Windows packaging completed successfully!" -ForegroundColor Green
