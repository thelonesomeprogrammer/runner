# runner

A Wayland-native application launcher written in Rust.

## What is this?

Runner displays a search interface on your Wayland desktop where you can type to find and launch programs. It searches through desktop applications, executables in your PATH, custom scripts, and previously-run commands.

Think of it as your keyboard-driven command palette for launching anything on your system.

## Why Runner?

- **Wayland-first**: Uses wlr-layer-shell protocol via smithay-client-toolkit
- **Group-based workflows**: Switch between curated lists of applications (work, media, development)
- **Static items**: Pin frequently-used commands with custom names in your config
- **Fuzzy search**: Powered by nucleo-matcher for intelligent text matching
- **Icon rendering**: Supports PNG, JPEG, ICO, and SVG icons via resvg

## Quick Start

Build and run:

```bash
cargo build --release
./target/release/runner
```

Or install to your system:

```bash
cargo install --path .
```

Launch a specific group:

```bash
runner --group media
```

## Configuring Runner

Configuration file: `~/.config/runner/config.toml`

Start by copying the example:

```bash
mkdir -p ~/.config/runner
cp example_config.toml ~/.config/runner/config.toml
```

### Understanding Groups

Groups let you create different "modes" for the launcher. Each group specifies:

- Which sources to search (desktop files, binaries, scripts, history)
- Filter patterns (whitelist or blacklist using regex)
- Environment variables to inject when launching
- Static items (hardcoded entries)

Example - a gaming group:

```toml
[groups.gaming]
sources = ["desktop"]
whitelist = ["steam", "lutris", "heroic"]
env = { MANGOHUD = "1", GAMEMODE = "1" }
```

Launch it with: `runner --group gaming`

### Static Items

Add custom entries that don't come from scanning:

```toml
[[groups.default.items]]
name = "System Monitor"
command = "kitty -e htop"
terminal = false
icon = "utilities-system-monitor"
```

These appear alongside scanned entries and respect fuzzy matching.

### Theme Options

Visual customization lives under `[theme]`:

```toml
[theme]
width = 600
height = 400
padding = 20.0
spacing = 10.0
border_radius = 12.0
background = "1e1e1eff"      # RRGGBBAA hex
text = "c8c8c8ff"
selection_background = "3c3c50ff"
selection_text = "ffffffff"
```

Color format: 8-character hex strings where the last two digits control transparency.

## How It Works

1. **Initialization**: Loads config and connects to Wayland compositor
2. **Source scanning**: Spawns background thread to collect entries from configured sources
3. **Event loop**: Runs calloop with Wayland events and channel receivers
4. **User input**: Updates search query, triggers fuzzy matching via nucleo
5. **Rendering**: Draws results using tiny-skia on a shared memory buffer
6. **Launch**: Forks and execs the selected command with configured environment

See `ARCHITECTURE.md` for module breakdown and design rationale.

## Keyboard Controls

When runner window is active:

- Type to search
- Arrow keys or Ctrl+N/P to navigate results
- Enter to launch selected entry
- Escape to close without launching

## Extending Runner

### Add a new source type

Create a file in `src/sources/` and implement the `Source` trait:

```rust
use crate::sources::Source;
use crate::model::{Entry, EntryType};

pub struct CustomSource;

impl Source for CustomSource {
    fn scan(&self) -> anyhow::Result<Vec<Entry>> {
        let entries = vec![
            Entry::new(
                "custom-id".into(),
                "Display Name".into(),
                "command --to-run".into(),
                EntryType::Custom,
                false, // terminal
            )
        ];
        Ok(entries)
    }
}
```

Then register it in `main.rs` within the source loading thread.

### Source Types

**Desktop**: Parses `.desktop` files from XDG application directories  
**Bin**: Lists executables found in PATH  
**Scripts**: Scans custom directories for executable scripts  
**History**: Recently launched commands (future feature)

## Development

Run with logging enabled:

```bash
RUST_LOG=debug cargo run
```

Format and lint:

```bash
cargo fmt
cargo clippy
```

## Project Structure

```
src/
├── main.rs          - Entry point, event loop setup
├── config.rs        - TOML parsing and data structures
├── state.rs         - Application state management
├── model.rs         - Entry and data types
├── matcher.rs       - Fuzzy matching wrapper
├── executor.rs      - Process spawning
├── sources/
│   ├── mod.rs       - Source trait
│   ├── desktop.rs   - XDG desktop file parser
│   ├── bin.rs       - PATH scanner
│   ├── scripts.rs   - Script directory scanner
│   └── history.rs   - Command history
└── ui/
    ├── wayland.rs   - Wayland protocol handling
    ├── render.rs    - Drawing logic with tiny-skia
    └── icons.rs     - Icon loading and caching
```

## Dependencies

Key crates used:

- `smithay-client-toolkit` - Wayland client library
- `calloop` - Event loop with Wayland integration
- `tiny-skia` - CPU-based 2D rendering
- `cosmic-text` - Text shaping and layout
- `nucleo-matcher` - Fast fuzzy matching
- `resvg` - SVG rendering
- `clap` - CLI argument parsing
- `serde` + `toml` - Configuration
