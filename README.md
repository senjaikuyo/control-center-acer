# Acer Control Center 3.0 (Rust Linux Edition)

A high-performance, lightweight, and native Linux replication of **Acer Control Center 3.0** designed for Acer Aspire 7 (A715 series), Acer ALG series, and Clevo-based laptops.

Reverse-engineered from original Acer Control Center 3.0 Windows binaries using **REA (`morluto/rea`)** and rewritten 100% in **Rust** with hardware-accelerated immediate mode graphics (`eframe` / `egui`).

---

## Features

- **Acer Control Center 3.0 Exact Visual Layout:**
  - Dark cyberpunk aesthetic with Acer cyan (`#00d2ff`) accents.
  - Circular dual-tachometer gauges with needle and speed track for CPU & GPU fans.
  - Live temperature badges and per-core clock frequency readouts.
- **Power & Thermal Modes (1-Click):**
  - 🌿 **Quiet:** Power-saver, silent fan acoustics.
  - ⚖️ **Entertainment / Balanced:** Balanced power profile for daily use.
  - ⚡ **Performance:** High boost CPU clock with aggressive cooling.
  - 🚀 **Turbo (Max Fan):** 100% full fan blast for intense gaming/rendering.
- **Dual Fan Speed Control:**
  - **Auto Curve:** Follows Clevo firmware / tailord profile.
  - **Manual Slider:** Override CPU (Fan 0) and GPU (Fan 1) speed directly from 20% to 100%.
- **Zero-Overhead Native Rust:**
  - Self-contained binary (< 9 MB).
  - Memory footprint: ~12 MB RAM.
  - Native Wayland (Hyprland, Niri, KDE Plasma) and X11 support.

---

## Build & Installation

```bash
cargo build --release
sudo cp target/release/acer-control-center-rust /usr/local/bin/acer-control-center
```

Or run via user space:
```bash
./install.sh
```
