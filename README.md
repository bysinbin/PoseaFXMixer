# Posea Audio 🎵

<p align="center">
  <img src="assets/icon.svg" width="128" height="128" alt="Posea Audio Logo" />
</p>

<p align="center">
  <b>High-Performance Real-Time Audio Enhancer, 31-Band Parametric Equalizer & Virtual Mixer</b>
  <br />
  <i>Open-source, cross-platform, ultra-low-latency sound enhancement engineered in Rust.</i>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust_2024-orange.svg" alt="Rust 2024" />
  <img src="https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-blue.svg" alt="Platform Support" />
  <img src="https://img.shields.io/badge/GUI-egui_0.37-red.svg" alt="egui GUI" />
  <img src="https://img.shields.io/badge/Audio-WASAPI%20%2F%20ALSA%20%2F%20CoreAudio-green.svg" alt="Audio Engine" />
  <img src="https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg" alt="License" />
</p>

---

## ✨ Features

- **31-Band ISO 1/3 Octave Graphic Equalizer**: Precise audio frequency tuning across 31 industry-standard ISO center frequencies (20 Hz – 20 kHz) with Catmull-Rom smooth cubic spline interpolation.
- **5 Core Enhancement Sliders**:
  - **Netlik (Clarity Exciter)**: High-frequency transient excitation and harmonic air.
  - **Ortam (Ambience)**: Psychoacoustic spatial Haas-delay room expansion.
  - **Çevresel Ses (Surround)**: Mid-side matrix stereo field widening.
  - **Dinamik Güçlendirme (Dynamic Boost)**: Multi-band RMS dynamics expander.
  - **Bass Güçlendirme (Bass Boost)**: Resonant sub-bass low-shelf punch.
- **Bipolar Symmetrical Spectrum Visualizer**: Real-time FFT audio spectrum analyzer with logarithmic frequency scaling and responsive falloff damping.
- **Ultra-Low Latency & Zero Void**: Custom frameless dark UI, 100% responsive resizing from all 8 directions, no black voids, fully alpha-blended transparent corners.
- **Virtual Audio Driver Bridge**: Automatic capture loopback from Posea Virtual Sink to physical speakers with automatic 100% hardware volume lock.
- **Presets Engine**: Studio-crafted sound curves for Music, Gaming, Voice/Podcasts, Movies, Streaming Video, Volume Boost, and Bass Boost.

---

## 🚀 Downloads & Installation

Pre-built binaries and installers are published under [GitHub Releases](https://github.com/bysinbin/PoseaFXMixer/releases).

### 🪟 Windows
- **Installer (.msi)**: Download `PoseaAudio-vX.X.X-Windows-x64.msi` and run the setup wizard.
- **Portable (.zip)**: Download `PoseaAudio-vX.X.X-Windows-x64-Portable.zip`, extract anywhere, and launch `PoseaAudio.exe`.

### 🍎 macOS
- **Disk Image (.dmg)**: Download `PoseaAudio-vX.X.X-macOS.dmg`, open it, and drag `Posea Audio.app` into your `Applications` folder.

### 🐧 Linux
- **Debian / Ubuntu (.deb)**:
  ```bash
  sudo dpkg -i posea-audio_1.0.0_amd64.deb
  ```
- **Tarball (.tar.gz)**:
  ```bash
  tar -xzf PoseaAudio-v1.0.0-linux-x64.tar.gz
  cd posea-audio-v1.0.0-linux-x64
  sudo ./install.sh
  ```

---

## 🛠️ Building from Source

### Prerequisites
- [Rust](https://rustup.rs/) (1.80+ or latest stable)
- Python 3 with `Pillow` (for icon generation):
  ```bash
  pip install Pillow
  ```

### Build Commands

```bash
# 1. Clone repository
git clone https://github.com/bysinbin/PoseaFXMixer.git
cd PoseaFXMixer

# 2. Generate icons (PNG, ICO, ICNS, SVG)
python scripts/generate_icons.py

# 3. Build optimized release executable
cargo build --release

# Run executable
./target/release/posea_fx_mixer
```

---

## 📁 Project Structure

```
├── assets/                    # Official icon assets (PNG, ICO, ICNS, SVG)
├── packaging/
│   ├── windows/posea_audio.wxs # WiX MSI installer configuration
│   ├── macos/Info.plist       # macOS App Bundle property list
│   └── linux/                 # Linux .desktop and Debian control files
├── scripts/
│   ├── generate_icons.py      # Multi-resolution icon builder
│   ├── package_windows.ps1    # Windows ZIP & MSI packaging
│   ├── package_macos.sh       # macOS .app & .dmg packaging
│   └── package_linux.sh       # Linux .deb & .tar.gz packaging
├── src/
│   ├── dsp/                   # 31-band EQ, effects, limiter, FFT visualizer
│   ├── engine/                # WASAPI audio bridge & driver management
│   ├── gui/                   # Egui frameless UI, sliders, custom widgets
│   └── main.rs                # Application entrypoint & viewport setup
├── .github/workflows/
│   └── release.yml            # Multi-platform CI/CD release workflow
└── Cargo.toml                 # Package manifest & dependencies
```

---

## 📄 License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
