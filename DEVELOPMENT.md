# Developer Guide

This document is for developers working on runner itself.

## Setting Up Development

### Requirements

- Rust toolchain 1.70+ (with edition 2024 support)
- Wayland compositor running
- Build dependencies:
  - libwayland-client
  - libxkbcommon

### Initial Setup

```bash
git clone <repo-url>
cd runner
cargo build
```

## Making Changes

### Code Organization

The codebase follows module-based organization:

**Core Logic**
- `config.rs`: Config file parsing, defaults
- `state.rs`: AppState that holds entries and selection
- `model.rs`: Entry struct and enums

**Data Sources**
- `sources/mod.rs`: Source trait definition
- `sources/desktop.rs`: .desktop file parsing
- `sources/bin.rs`: PATH scanning
- `sources/scripts.rs`: Script directory scanning
- `sources/history.rs`: Command history tracking

**UI Layer**
- `ui/wayland.rs`: Wayland protocol, window management
- `ui/render.rs`: Drawing with tiny-skia
- `ui/icons.rs`: Icon loading, SVG/raster support

**Utilities**
- `matcher.rs`: Fuzzy matching via nucleo
- `executor.rs`: Command execution

### Testing Changes

Since runner is a graphical Wayland application, most testing is manual:

1. Run with debug output:
   ```bash
   RUST_LOG=runner=debug cargo run
   ```

2. Test specific group:
   ```bash
   cargo run -- --group dev
   ```

3. Check for common issues:
   ```bash
   cargo clippy
   cargo fmt --check
   ```

### Adding a Source

To implement a new data source:

1. Create `src/sources/newsource.rs`
2. Implement the `Source` trait
3. Update `main.rs` to check for source name in group config
4. Add documentation to example_config.toml

Example skeleton:

```rust
use crate::sources::Source;
use crate::model::{Entry, EntryType};
use anyhow::Result;

pub struct NewSource {
    // config fields if needed
}

impl Source for NewSource {
    fn scan(&self) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();
        
        // Your scanning logic here
        // Create Entry objects
        
        Ok(entries)
    }
}
```

Register in `main.rs`:

```rust
if sources_to_scan.contains(&"newsource".to_string()) {
    if let Ok(mut e) = NewSource.scan() {
        entries.append(&mut e);
    }
}
```

### Modifying Rendering

Rendering happens in `src/ui/render.rs`. The `Renderer` struct handles:

- Text layout with cosmic-text
- Drawing shapes with tiny-skia
- Icon compositing

Key functions:
- `draw()`: Main rendering entry point
- `draw_input_box()`: Search box rendering
- `draw_results()`: Result list with icons

When modifying rendering:
1. Check theme config values in `config.rs`
2. Test with different window sizes
3. Verify text doesn't overflow
4. Test with and without icons

### Configuration Changes

When adding new config options:

1. Add field to relevant struct in `config.rs` with `#[serde(default)]`
2. Implement `Default` or use `#[serde(default = "function_name")]`
3. Use the field in relevant code
4. Document in `example_config.toml`

Example:

```rust
#[derive(Deserialize, Debug, Clone)]
pub struct ThemeConfig {
    #[serde(default = "default_font_size")]
    pub font_size: f32,
    // ... other fields
}

fn default_font_size() -> f32 { 14.0 }
```

## Code Style

### Rust Conventions

- Follow standard rustfmt style (run `cargo fmt`)
- Avoid `unwrap()` in production code paths - use `?` or handle errors
- Use `#[allow(dead_code)]` sparingly and only when necessary
- Prefer explicit types when they improve clarity

### Error Handling

Use `anyhow::Result` for fallible operations:

```rust
use anyhow::{Result, Context};

fn load_something() -> Result<Data> {
    let file = fs::read_to_string(path)
        .context("Failed to read config file")?;
    // ...
    Ok(data)
}
```

### Logging

Use the `log` crate with appropriate levels:

```rust
use log::{debug, info, warn, error};

debug!("Scanning directory: {}", path);
info!("Loaded {} entries", count);
warn!("Icon not found: {}", name);
error!("Failed to connect to Wayland: {}", e);
```

## Architecture Decisions

### Why Calloop?

calloop integrates well with Wayland event sources and provides a single-threaded event loop that fits runner's simple execution model.

### Why Tiny-skia?

CPU-based rendering is sufficient for runner's simple UI. tiny-skia is pure Rust, has no system dependencies, and performs well for our use case.

### Why Nucleo?

nucleo-matcher is battle-tested (used in Helix editor) and provides the same fuzzy matching behavior as fzf.

## Common Tasks

### Changing Theme Defaults

Edit functions in `config.rs`:

```rust
fn default_width() -> u32 { 600 }
fn default_height() -> u32 { 400 }
```

### Adding a CLI Flag

Use clap derive in `main.rs`:

```rust
#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    new_option: String,
}
```

### Icon Lookup Paths

Icon resolution logic is in `ui/icons.rs`. It follows freedesktop icon theme spec:
1. Check theme directories
2. Fall back to hicolor
3. Support PNG, SVG, ICO, JPEG

## Performance Considerations

### Source Scanning

Desktop and bin sources run in a background thread to avoid blocking the UI. Heavy I/O should stay off the main thread.

### Fuzzy Matching

Nucleo scoring happens synchronously when the query changes. For very large entry lists (>10k), this could impact UI responsiveness. Current implementation is fine for typical use.

### Icon Loading

Icons are loaded asynchronously via channels. SVGs are rasterized once and cached.

## Debugging Tips

### Wayland Issues

Set these environment variables:

```bash
WAYLAND_DEBUG=1 cargo run
```

### Memory Leaks

Run with valgrind (though Rust's safety guarantees make this rare):

```bash
cargo build
valgrind ./target/debug/runner
```

### Render Issues

tiny-skia renders to a buffer. To debug drawing:
1. Add debug output in `render.rs`
2. Check pixmap dimensions match window size
3. Verify colors are in correct RGBA format

## Release Process

When cutting a release:

1. Update version in `Cargo.toml`
2. Update ARCHITECTURE.md if needed
3. Tag the release
4. Build release binary: `cargo build --release --locked`

## Getting Help

- Check existing code for patterns
- Read ARCHITECTURE.md for design overview
- Look at dependencies' documentation
- Open an issue for questions
