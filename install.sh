#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"

echo "=== Installing Acer Control Center for Linux ==="

# 1. Create wrapper in ~/.local/bin
mkdir -p "$HOME/.local/bin"
cat << EOF > "$HOME/.local/bin/acer-control-center"
#!/usr/bin/env bash
exec /usr/bin/python3 "$DIR/src/main.py" "\$@"
EOF
chmod +x "$HOME/.local/bin/acer-control-center"
echo "Installed CLI launcher: $HOME/.local/bin/acer-control-center"

# 2. Install desktop launcher & icon
mkdir -p "$HOME/.local/share/applications"
mkdir -p "$HOME/.local/share/icons/hicolor/128x128/apps"

cp "$DIR/assets/icon.png" "$HOME/.local/share/icons/hicolor/128x128/apps/acer-control-center.png"

cat << EOF > "$HOME/.local/share/applications/com.acer.ControlCenterLinux.desktop"
[Desktop Entry]
Name=Acer Control Center
Comment=Acer Control Center for Linux (Fan Control & Power Modes)
Exec=$HOME/.local/bin/acer-control-center
Icon=acer-control-center
Terminal=false
Type=Application
Categories=System;Settings;HardwareSettings;
Keywords=acer;fan;speed;control;center;turbo;clevo;temperature;
StartupNotify=true
EOF

update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true

echo "Installation finished! You can now launch 'Acer Control Center' from your app launcher or run 'acer-control-center' in terminal."
