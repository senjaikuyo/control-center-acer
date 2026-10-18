#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"

echo "=== Building & Installing Acer Control Center (Rust) ==="

cargo build --release --manifest-path "$DIR/Cargo.toml"

mkdir -p "$HOME/.local/bin"
install -m 755 "$DIR/target/release/acer-control-center-rust" "$HOME/.local/bin/acer-control-center"

mkdir -p "$HOME/.local/share/icons/hicolor/64x64/apps" "$HOME/.local/share/pixmaps"
if [ -f "$DIR/assets/acer-control-center.png" ]; then
    install -m 644 "$DIR/assets/acer-control-center.png" "$HOME/.local/share/icons/hicolor/64x64/apps/acer-control-center.png"
    install -m 644 "$DIR/assets/acer-control-center.png" "$HOME/.local/share/pixmaps/acer-control-center.png"
fi

mkdir -p "$HOME/.local/share/applications"
cat << EOF > "$HOME/.local/share/applications/com.acer.ControlCenterLinux.desktop"
[Desktop Entry]
Name=Acer Control Center
GenericName=Hardware Control Center
Comment=Acer / Clevo Control Center 3.0 for Linux (Rust Edition)
Exec=$HOME/.local/bin/acer-control-center
Icon=acer-control-center
Terminal=false
Type=Application
Categories=System;Settings;HardwareSettings;
Keywords=acer;fan;speed;control;center;turbo;clevo;temperature;rgb;keyboard;
StartupNotify=true
StartupWMClass=com.clevo.ControlCenter
EOF

update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true
gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true

echo "Installation complete! Launch via app menu or run 'acer-control-center'."
