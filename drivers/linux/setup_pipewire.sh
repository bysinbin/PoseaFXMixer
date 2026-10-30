#!/usr/bin/env bash
# PoseaFXMixer - Linux PipeWire / PulseAudio Virtual Audio Sink Kurulumu
set -e

echo "=== Posea Audio Virtual Sink Kuruluyor ==="

# 1. Null-sink modülünü yükle
MODULE_ID=$(pactl load-module module-null-sink sink_name=posea_audio sink_properties=device.description="Posea_Audio_Virtual_Device")

echo "Posea Audio başarıyla yüklendi! Modül ID: $MODULE_ID"
echo "Kaldırmak için: pactl unload-module $MODULE_ID"
