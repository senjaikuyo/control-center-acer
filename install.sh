#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"

echo "=== Building & Installing Acer Control Center (Rust) ==="

cargo build --release --manifest-path "$DIR/Cargo.toml"

mkdir -p "$HOME/.local/bin"
cp "$DIR/target/release/acer-control-center-rust" "$HOME/.local/bin/acer-control-center"
chmod +x "$HOME/.local/bin/acer-control-center"

mkdir -p "$HOME/.local/share/applications"
cat << EOF > "$HOME/.local/share/applications/com.acer.ControlCenterLinux.desktop"
[Desktop Entry]
Name=Acer Control Center
Comment=Acer Control Center for Linux (Rust Edition)
Exec=$HOME/.local/bin/acer-control-center
Icon=acer-control-center
Terminal=false
Type=Application
Categories=System;Settings;HardwareSettings;
Keywords=acer;fan;speed;control;center;turbo;clevo;temperature;
StartupNotify=true
EOF

update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true

echo "Installation complete! Launch via app menu or run 'acer-control-center'."
