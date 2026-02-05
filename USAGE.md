# Usage Guide

Practical examples for using runner in everyday workflows.

## Basic Usage

### Launching Runner

Start runner with your configured default group:

```bash
runner
```

The launcher window appears. Start typing to search.

### Switching Groups

Launch with a specific group:

```bash
runner --group media
```

This activates the "media" group from your config, which might only show media players and related apps.

## Configuration Patterns

### Example: Work vs Personal

Separate work apps from personal ones:

```toml
[groups.work]
sources = ["desktop"]
whitelist = ["slack", "teams", "outlook", "chrome"]
env = { WORK_MODE = "1" }

[groups.personal]
sources = ["desktop", "bin"]
blacklist = ["slack", "teams", "outlook"]
```

Launch for work: `runner --group work`  
Launch for personal: `runner --group personal`

### Example: Quick Commands

Pin frequently-used terminal commands:

```toml
[[groups.default.items]]
name = "Update System"
command = "kitty -e sudo pacman -Syu"
terminal = false

[[groups.default.items]]
name = "Edit Hosts"
command = "sudo nvim /etc/hosts"
terminal = true
```

These appear in search results alongside other entries.

### Example: Development Environment

Set environment variables for dev tools:

```toml
[groups.dev]
sources = ["desktop", "bin"]
whitelist = ["code", "kitty", "firefox", "chromium", "git"]
env = { 
    RUST_BACKTRACE = "1",
    NODE_ENV = "development",
    DEBUG = "*"
}
```

Apps launched through this group inherit these variables.

### Example: Gaming Setup

Inject performance tools:

```toml
[groups.gaming]
sources = ["desktop"]
whitelist = ["steam", "lutris", "heroic", "discord"]
env = {
    MANGOHUD = "1",
    GAMEMODE = "1",
    DXVK_HUD = "fps"
}
```

### Example: Blacklist Dangerous Commands

Prevent accidental launches:

```toml
[groups.default]
sources = ["desktop", "bin", "history"]
blacklist = [
    "^rm$",
    "^reboot$",
    "^poweroff$",
    "^shutdown$",
    "dd",
    "mkfs"
]
```

Uses regex patterns. `^rm$` matches exactly "rm", while "dd" matches anything containing "dd".

## Theme Customization

### Dark Theme

```toml
[theme]
width = 700
height = 500
background = "1a1a1aff"
border_color = "2a2a2aff"
text = "e0e0e0ff"
selection_background = "404040ff"
selection_text = "ffffffff"
```

### Light Theme

```toml
[theme]
background = "f5f5f5ff"
border_color = "d0d0d0ff"
text = "202020ff"
selection_background = "e0e0e0ff"
selection_text = "000000ff"
```

### High Contrast

```toml
[theme]
background = "000000ff"
text = "ffffffff"
selection_background = "ffffffff"
selection_text = "000000ff"
```

### Transparent Background

Use alpha channel (last two hex digits):

```toml
[theme]
background = "1e1e1ecc"  # 80% opacity
```

## Integration Examples

### Keybinding with Sway

Add to your Sway config:

```
bindsym $mod+d exec runner
bindsym $mod+Shift+d exec runner --group work
```

### Keybinding with Hyprland

```
bind = SUPER, D, exec, runner
bind = SUPER SHIFT, D, exec, runner --group dev
```

### Systemd User Service

Auto-start on login (if needed):

```ini
[Unit]
Description=Runner application launcher
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/local/bin/runner
Restart=on-failure

[Install]
WantedBy=default.target
```

Enable: `systemctl --user enable --now runner.service`

*Note: Runner typically shouldn't run as a service; launch it on-demand with keybindings instead.*

## Tips and Tricks

### Fuzzy Matching

You don't need to type the full name. Runner matches on:
- Any characters in order: "frf" matches "Firefox"
- Acronyms: "gc" matches "Google Chrome"
- Word boundaries: "fire fox" matches "Firefox"

### History Priority

Frequently-used commands appear higher in results (when history source is enabled).

### Terminal Apps

Set `terminal = true` in desktop files or static items to auto-launch in your configured terminal emulator.

### Icon Paths

Icons can be:
- Theme icon names: "firefox", "code"
- Absolute paths: "/usr/share/pixmaps/app.png"
- User paths: "~/.local/share/icons/custom.svg"

### Multiple Sources

Combine sources for comprehensive coverage:

```toml
[groups.everything]
sources = ["desktop", "bin", "scripts", "history"]
```

### Source-Specific Groups

Desktop apps only:

```toml
[groups.apps]
sources = ["desktop"]
```

Command-line tools only:

```toml
[groups.cli]
sources = ["bin"]
```

## Troubleshooting

### No Window Appears

Check if Wayland is running:
```bash
echo $WAYLAND_DISPLAY
```

Should output something like `wayland-0` or `wayland-1`.

### Icons Not Loading

Verify icon themes are installed:
```bash
ls /usr/share/icons/
```

Runner looks in standard icon directories.

### Config Not Loading

Check config file location:
```bash
cat ~/.config/runner/config.toml
```

Run with logging to see config parsing:
```bash
RUST_LOG=runner=debug runner
```

### Search Not Finding Apps

Check which sources are enabled in your active group:
```bash
runner --group default
```

Verify the group's `sources` list includes what you expect.

## Advanced Patterns

### Context-Aware Launching

Different groups for different contexts:

```bash
#!/bin/bash
# launch-runner.sh
if [[ $(hostname) == "work-laptop" ]]; then
    runner --group work
else
    runner --group personal
fi
```

Bind this script to your launcher key.

### Dynamic Whitelists

You can generate config programmatically:

```bash
# Generate a group with currently running apps
ps aux | awk '{print $11}' | sort -u > /tmp/running.txt
# Then reference in whitelist generation script
```

### Profile Switching

Create multiple config files and symlink:

```bash
ln -sf ~/.config/runner/work-config.toml ~/.config/runner/config.toml
```

Switch based on time of day, location, etc.
