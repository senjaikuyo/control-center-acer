# Acer Control Center (Linux Edition)

A lightweight, modern, and native Linux replication of **Acer Control Center 3.0** designed for Acer Aspire 7 (A715 series), Acer ALG series, and Clevo-based gaming laptops on Arch Linux and other distributions.

Built with **Python 3 + GTK4 / Libadwaita** and integrates with the Clevo/Tuxedo hardware D-Bus interface.

---

## Features

- **Acer Control Center 3.0 Dark Gaming Aesthetics:** Neon cyan accents, card-based layout, and sleek hardware gauges.
- **Dual Fan Speed Control:**
  - Automatic firmware curve mode.
  - Manual sliders for CPU (Fan 0) and GPU (Fan 1) (20% – 100%).
  - 1-Click **Turbo Mode** (100% full fan blast for heavy gaming and rendering).
- **Power & Thermal Modes:**
  - 🌿 **Quiet / Eco** (Power saver, silent acoustics)
  - ⚖️ **Balanced** (Optimal day-to-day work & battery)
  - ⚡ **Performance** (High CPU frequency scaling)
  - 🚀 **Turbo** (Max boost & max fans)
- **Live Hardware Monitoring (Zero Lag):**
  - Real-time CPU per-core temperature & clock frequency.
  - Real-time NVIDIA GPU temperature via direct NVML (C library, 0.1ms latency).
- **Ultra Lightweight:**
  - Consumes only ~15–20 MB RAM.
  - Instant launch (<0.2s).
  - Native Wayland (Hyprland, Niri, KDE Plasma) and X11 support.

---

## Hardware Compatibility

Specifically tested and optimized for:
- **Acer Aspire 7 (A715-59G, A715-79G)** (Intel 13th Gen i5-13420H + NVIDIA RTX 3050)
- **Acer ALG Series**
- Any laptop using **Clevo internal motherboard hardware** with `CLV0001` ACPI.

---

## Prerequisites (Arch Linux)

Ensure `tuxedo-drivers-dkms` and `tailord` are installed and running:

```bash
# 1. Add repository to /etc/pacman.conf
[tuxedo-repo]
SigLevel = Optional TrustAll
Server = https://raw.githubusercontent.com/atul977/Aspire-A715-79G-LinuxPackages/gh-pages/

# 2. Install packages
sudo pacman -Syy tuxedo-drivers-dkms tailord

# 3. Enable daemon
sudo systemctl enable --now tailord.service
```

---

## Installation

```bash
git clone https://github.com/<your-username>/acer-control-center-linux.git
cd acer-control-center-linux
./install.sh
```

Launch it from your app launcher (search **Acer Control Center**) or run:
```bash
acer-control-center
```

---

## License

MIT License.
