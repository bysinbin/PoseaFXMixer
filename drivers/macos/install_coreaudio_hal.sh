#!/usr/bin/env bash
# PoseaFXMixer - macOS CoreAudio HAL AudioServerPlugIn Kurulumu
set -e

echo "=== Posea Audio CoreAudio HAL Sürücüsü Kuruluyor ==="

PLUGIN_DIR="/Library/Audio/Plug-Ins/HAL"
DRIVER_NAME="PoseaAudio.driver"

if [ ! -d "$DRIVER_NAME" ]; then
    echo "Hata: $DRIVER_NAME dizini bulunamadı!"
    exit 1
fi

sudo cp -R "$DRIVER_NAME" "$PLUGIN_DIR/"
echo "CoreAudio servisi yeniden başlatılıyor..."
sudo killall coreaudiod

echo "Kurulum tamamlandı! macOS Ses Tercihleri'nde 'Posea Audio' görünecektir."
