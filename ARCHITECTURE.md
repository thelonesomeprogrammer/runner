# Architecture: Runner (Rust + Wayland)

## 1. High-Level Architecture

The application follows a unidirectional data flow architecture centered around an event loop (`calloop`).

1.  **Core/State**: Holds the application state (loaded entries, current search query, selection index, configuration).
2.  **Input/Source Manager**: Asynchronously loads entries from configured sources (`.desktop` files, binaries, history).
3.  **UI/Renderer**: A Wayland client (using `smithay-client-toolkit`) that renders the state to a shared memory buffer (using `tiny-skia` or `cairo`) and handles keyboard input.
4.  **Executor**: Handles the final execution of the selected entry, managing process forking, environment variables, and container contexts.

## 2. Rust Module Layout

```
src/
├── main.rs           # Entry point, CLI parsing, Event loop init
├── config.rs         # TOML loading, XDG resolution, Struct defs
├── state.rs          # AppState, Selection logic
├── sources/
│   ├── mod.rs        # Source trait definition
│   ├── desktop.rs    # XDG .desktop file parser
│   ├── bin.rs        # $PATH binary scanner
│   └── history.rs    # Recently used commands
├── model.rs          # Core data structures (Entry, LaunchGroup)
├── ui/
│   ├── mod.rs
│   ├── wayland.rs    # SCTK integration, Window setup
│   └── render.rs     # Drawing logic (Text, Rects)
├── matcher.rs        # Fuzzy matching logic (nucleo-matcher wrapper)
└── executor.rs       # Process spawning & Container handling
```

## 3. Core Data Models

```rust
// model.rs

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: String,            // Unique ID (e.g., "firefox.desktop")
    pub name: String,          // Display name
    pub command: String,       // Executable command
    pub icon: Option<String>,  // Icon name/path
    pub score: u16,            // Fuzzy match score
    pub group: String,         // The launch group it belongs to
    pub is_container: bool,    // Context hint
}

// config.rs

#[derive(Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub groups: HashMap<String, LaunchGroup>,
    #[serde(default)]
    pub sources: SourceConfig,
}

#[derive(Deserialize)]
pub struct LaunchGroup {
    pub sources: Vec<String>, // ["desktop", "bin"]
    pub env: Option<HashMap<String, String>>,
    pub blacklist: Option<Vec<String>>,
}
```

## 4. Example `config.toml`

```toml
# ~/.config/runner/config.toml

[general]
history_size = 50
terminal = "alacritty -e"

[sources]
# Global source settings
scan_path = true
scan_desktop = true

[groups.default]
sources = ["history", "desktop", "bin"]
blacklist = ["^rm$", "poweroff"]

[groups.dev]
sources = ["desktop"]
# Only show dev tools
whitelist = ["code", "alacritty", "git-gui"]
env = { RUST_BACKTRACE = "1" }

[groups.media]
sources = ["desktop"]
whitelist = ["spotify", "vlc", "mpv"]

[container.arch]
# Detect if we are running in a blendOS/distrobox container
check_env = "CONTAINER_ID=arch"
command_prefix = "distrobox-enter -n arch -- "
```

## 5. Execution Flow

1.  **Bootstrap**: `main` initializes `calloop`, loads `Config` (parsing CLI args for overrides like `--group dev`).
2.  **Window**: Initialize Wayland window. If Wayland is missing, panic (per constraints).
3.  **Load**: `SourceManager` spawns threads/tasks to scan `$PATH` and `$XDG_DATA_DIRS`.
4.  **Loop**:
    *   **Wait**: Block for Wayland events or Source results.
    *   **Input**: User types -> Update `query` -> Trigger `Matcher`.
    *   **Match**: Filter `Entry` list against `query` using fuzzy algorithm. Sort by score.
    *   **Render**: Draw background, input box, and list of results.
5.  **Action**: User presses `Enter`.
    *   Identify selected `Entry`.
    *   Pass to `Executor`.
    *   `Executor` applies Group env vars and Container prefixes.
    *   `Command::spawn()` and detach.
    *   App exits.

## 6. Recommended Rust Crates

*   **Config**: `serde`, `serde_derive`, `toml`, `directories` (XDG).
*   **Wayland**: `smithay-client-toolkit` (SCTK), `wayland-client`.
*   **Drawing**: `tiny-skia` (CPU rendering, fast, safe) or `cairo-rs`. `cosmic-text` (Text layout/shaping).
*   **Fuzzy**: `nucleo-matcher` (Rust rewrite of fzf logic, extremely fast).
*   **CLI**: `clap`.
*   **Async/Loop**: `calloop` (integrates well with SCTK).

## 7. Minimal MVP Scope

1.  **Skeleton**: Binary that reads a basic TOML config.
2.  **Wayland Window**: Opens a blank window on screen.
3.  **Input**: Accepts keyboard input (text + arrows).
4.  **Source**: Reads only `.desktop` files from `/usr/share/applications`.
5.  **Match**: Basic substring search.
6.  **Exec**: Launches the command via `std::process::Command`.

## 8. Clear Extension Points

*   **Source Trait**: `trait Source { fn scan(&self) -> Vec<Entry>; }`. New sources (e.g., browser bookmarks, calc) just implement this.
*   **Renderer Trait**: Decouple logic from `tiny-skia` to allow future GPU renderers if needed.
