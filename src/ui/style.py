"""
CSS Styles for Acer Control Center 3.0 (Linux Edition)
Faithful reproduction of Clevo / Acer CC3.0 dark gaming theme.
"""

CSS_STYLES = """
/* Window Base */
window.acer-window {
    background-color: #0b0f19;
    color: #e6edf3;
    font-family: 'Segoe UI', 'Inter', -apple-system, sans-serif;
}

/* Header Bar */
.acer-header {
    background: transparent;
    padding: 10px 16px;
    border-bottom: 1px solid rgba(0, 210, 255, 0.2);
}

.window-title {
    font-size: 15px;
    font-weight: 800;
    letter-spacing: 2.5px;
    color: #00d2ff;
    text-transform: uppercase;
}

.window-subtitle {
    font-size: 11px;
    color: #8b949e;
    font-weight: 500;
}

/* Window Control Buttons */
.win-ctrl-btn {
    background: transparent;
    border: none;
    border-radius: 4px;
    padding: 4px;
    min-width: 32px;
    min-height: 28px;
    transition: background 0.2s ease;
}

.win-ctrl-btn:hover {
    background-color: rgba(255, 255, 255, 0.08);
}

.win-close-btn:hover {
    background-color: #e81123;
}

/* Navigation Tabs */
.nav-tab-box {
    background-color: rgba(13, 17, 23, 0.7);
    border-right: 1px solid rgba(48, 54, 61, 0.5);
    padding: 12px 6px;
}

.nav-tab-btn {
    background: transparent;
    border: none;
    border-radius: 8px;
    padding: 12px 14px;
    color: #8b949e;
    font-size: 13px;
    font-weight: 600;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.nav-tab-btn:hover {
    color: #00d2ff;
    background-color: rgba(0, 210, 255, 0.08);
}

.nav-tab-btn.active {
    color: #ffffff;
    background: linear-gradient(90deg, rgba(0, 210, 255, 0.25) 0%, rgba(0, 210, 255, 0.05) 100%);
    border-left: 3px solid #00d2ff;
    box-shadow: inset 0 0 10px rgba(0, 210, 255, 0.2);
}

/* Power Mode Cards */
.mode-card {
    background-color: rgba(22, 27, 34, 0.85);
    border: 2px solid rgba(48, 54, 61, 0.8);
    border-radius: 14px;
    padding: 20px;
    transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.mode-card:hover {
    border-color: rgba(0, 210, 255, 0.5);
    background-color: rgba(22, 27, 34, 0.98);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6);
}

.mode-card.active-quiet {
    border-color: #2ea043;
    background: linear-gradient(145deg, rgba(46, 160, 67, 0.15) 0%, rgba(22, 27, 34, 0.9) 100%);
    box-shadow: 0 0 20px rgba(46, 160, 67, 0.35);
}

.mode-card.active-power-saving {
    border-color: #388bfd;
    background: linear-gradient(145deg, rgba(56, 139, 253, 0.15) 0%, rgba(22, 27, 34, 0.9) 100%);
    box-shadow: 0 0 20px rgba(56, 139, 253, 0.35);
}

.mode-card.active-entertainment {
    border-color: #d29922;
    background: linear-gradient(145deg, rgba(210, 153, 34, 0.15) 0%, rgba(22, 27, 34, 0.9) 100%);
    box-shadow: 0 0 20px rgba(210, 153, 34, 0.35);
}

.mode-card.active-performance {
    border-color: #f85149;
    background: linear-gradient(145deg, rgba(248, 81, 73, 0.2) 0%, rgba(22, 27, 34, 0.9) 100%);
    box-shadow: 0 0 24px rgba(248, 81, 73, 0.45);
}

.mode-title {
    font-size: 16px;
    font-weight: 800;
    letter-spacing: 1.5px;
    text-transform: uppercase;
}

.mode-desc {
    font-size: 12px;
    color: #8b949e;
    line-height: 1.4;
}

/* Fan Dashboards & Sliders */
.fan-card {
    background-color: rgba(22, 27, 34, 0.9);
    border: 1px solid rgba(48, 54, 61, 0.8);
    border-radius: 14px;
    padding: 24px;
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.4);
}

.fan-title {
    font-size: 14px;
    font-weight: 700;
    color: #00d2ff;
    letter-spacing: 1.5px;
    text-transform: uppercase;
}

.speed-value {
    font-size: 32px;
    font-weight: 900;
    color: #ffffff;
    font-feature-settings: "tnum";
}

.speed-unit {
    font-size: 14px;
    font-weight: 600;
    color: #8b949e;
    margin-bottom: 6px;
}

.temp-badge {
    background-color: rgba(0, 210, 255, 0.12);
    border: 1px solid rgba(0, 210, 255, 0.3);
    border-radius: 8px;
    padding: 4px 12px;
    font-size: 14px;
    font-weight: 700;
    color: #00d2ff;
}

/* Sliders */
scale.cyber-slider trough {
    background-color: #21262d;
    border-radius: 6px;
    min-height: 8px;
}

scale.cyber-slider highlight {
    background: linear-gradient(90deg, #00d2ff 0%, #388bfd 100%);
    border-radius: 6px;
    box-shadow: 0 0 8px rgba(0, 210, 255, 0.5);
}

scale.cyber-slider slider {
    background-color: #ffffff;
    border: 2px solid #00d2ff;
    border-radius: 50%;
    min-width: 22px;
    min-height: 22px;
    box-shadow: 0 0 10px rgba(0, 210, 255, 0.8);
    transition: transform 0.1s ease;
}

scale.cyber-slider slider:hover {
    transform: scale(1.15);
}

/* Turbo Button */
.turbo-btn {
    background: linear-gradient(135deg, #f85149 0%, #da3633 100%);
    border: none;
    border-radius: 10px;
    color: #ffffff;
    font-size: 14px;
    font-weight: 800;
    padding: 12px 24px;
    letter-spacing: 1.5px;
    box-shadow: 0 4px 16px rgba(248, 81, 73, 0.4);
    transition: all 0.2s ease;
}

.turbo-btn:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(248, 81, 73, 0.6);
}

.turbo-btn.active {
    background: linear-gradient(135deg, #ff7b72 0%, #f85149 100%);
    box-shadow: 0 0 25px #f85149;
    border: 2px solid #ffffff;
}

/* Action Buttons */
.action-btn {
    background-color: rgba(48, 54, 61, 0.5);
    border: 1px solid rgba(48, 54, 61, 0.8);
    border-radius: 8px;
    color: #c9d1d9;
    font-weight: 600;
    font-size: 13px;
    padding: 8px 18px;
    transition: all 0.2s ease;
}

.action-btn:hover {
    background-color: rgba(56, 139, 253, 0.15);
    border-color: #388bfd;
    color: #58a6ff;
}

.action-btn.active {
    background-color: rgba(0, 210, 255, 0.2);
    border-color: #00d2ff;
    color: #ffffff;
}
"""
