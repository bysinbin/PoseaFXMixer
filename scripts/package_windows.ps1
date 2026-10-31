# Posea Audio Windows Packaging Script
param(
    [string]$Version = "1.0.0"
)

$ErrorActionPreference = "Stop"

Write-Host ">>> [1/3] Building Posea Audio in Release Mode..." -ForegroundColor Cyan
Get-Process -Name "posea_fx_mixer*" -ErrorAction SilentlyContinue | Stop-Process -Force
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

$wixCandidates = @(
    "C:\Program Files (x86)\WiX Toolset v3.11\bin",
    "C:\Program Files (x86)\WiX Toolset v3.14\bin",
    "C:\Program Files\WiX Toolset v3.11\bin"
)
foreach ($dir in $wixCandidates) {
    if (Test-Path "$dir\candle.exe") {
        $env:PATH = "$dir;$env:PATH"
        break
    }
}

if ((Get-Command "candle.exe" -ErrorAction SilentlyContinue) -and (Get-Command "light.exe" -ErrorAction SilentlyContinue)) {
    try {
        Write-Host "    Compiling WiX MSI installer (x64)..." -ForegroundColor Yellow
        candle.exe -nologo -arch x64 -out "$distDir\windows\posea.wixobj" "packaging\windows\posea_audio.wxs"
        light.exe -nologo -ext WixUIExtension -out "$distDir\PoseaAudio-v$Version-Windows-x64.msi" "$distDir\windows\posea.wixobj"
        Write-Host "    MSI Installer created: $distDir\PoseaAudio-v$Version-Windows-x64.msi" -ForegroundColor Green
    } catch {
        Write-Warning "Failed to compile MSI installer: $_"
    }
} else {
    Write-Host "    WiX Toolset (candle/light) not found on PATH. Only portable ZIP generated." -ForegroundColor Yellow
}

Write-Host ">>> Windows packaging completed successfully!" -ForegroundColor Green
