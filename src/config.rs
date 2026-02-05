use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use directories::ProjectDirs;
use anyhow::Result;
use std::fs;
use tiny_skia::Color;

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub groups: HashMap<String, LaunchGroup>,
    #[serde(default)]
    pub sources: SourceConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct GeneralConfig {
    #[serde(default = "default_history_size")]
    pub history_size: usize,
    #[serde(default)]
    pub terminal: Option<String>,
}

fn default_history_size() -> usize { 50 }

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            history_size: default_history_size(),
            terminal: None,
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct SourceConfig {
    #[serde(default = "default_true")]
    pub scan_path: bool,
    #[serde(default = "default_true")]
    pub scan_desktop: bool,
    #[serde(default = "default_true")]
    pub scan_scripts: bool,
}

fn default_true() -> bool { true }

impl Default for SourceConfig {
    fn default() -> Self {
        Self {
            scan_path: true,
            scan_desktop: true,
            scan_scripts: true,
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone, Default)]
pub struct LaunchGroup {
    #[serde(default)]
    pub sources: Vec<String>,
    pub env: Option<HashMap<String, String>>,
    pub blacklist: Option<Vec<String>>,
    pub whitelist: Option<Vec<String>>,
    #[serde(default)]
    pub items: Vec<StaticEntry>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct StaticEntry {
    pub name: String,
    pub command: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub terminal: bool,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug, Clone)]
pub struct ThemeConfig {
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
    #[serde(default = "default_padding")]
    pub padding: f32,
    #[serde(default = "default_spacing")]
    pub spacing: f32,
    #[serde(default = "default_border_radius")]
    pub border_radius: f32,
    #[serde(default = "default_background")]
    pub background: String,
    #[serde(default = "default_border_color")]
    pub border_color: String,
    #[serde(default = "default_text")]
    pub text: String,
    #[serde(default = "default_selection_background")]
    pub selection_background: String,
    #[serde(default = "default_selection_text")]
    pub selection_text: String,
    #[serde(default = "default_number_color")]
    pub number_color: String,
}

fn default_width() -> u32 { 600 }
fn default_height() -> u32 { 400 }
fn default_padding() -> f32 { 20.0 }
fn default_spacing() -> f32 { 10.0 }
fn default_border_radius() -> f32 { 12.0 }
fn default_background() -> String { "1e1e1eff".to_string() }
fn default_border_color() -> String { "3c3c50ff".to_string() }
fn default_text() -> String { "c8c8c8ff".to_string() }
fn default_selection_background() -> String { "3c3c50ff".to_string() }
fn default_selection_text() -> String { "ffffffff".to_string() }
fn default_number_color() -> String { "646464ff".to_string() }

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            padding: default_padding(),
            spacing: default_spacing(),
            border_radius: default_border_radius(),
            background: default_background(),
            border_color: default_border_color(),
            text: default_text(),
            selection_background: default_selection_background(),
            selection_text: default_selection_text(),
            number_color: default_number_color(),
        }
    }
}

impl ThemeConfig {
    pub fn parse_color(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 8 {
            return Color::BLACK;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
        
        Color::from_rgba8(r, g, b, a)
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut groups = HashMap::new();
        groups.insert("default".to_string(), LaunchGroup {
            sources: vec!["desktop".to_string(), "bin".to_string(), "scripts".to_string()],
            env: None,
            blacklist: None,
            whitelist: None,
            items: vec![],
        });

        Self {
            general: GeneralConfig::default(),
            groups,
            sources: SourceConfig::default(),
            theme: ThemeConfig::default(),
        }
    }
}

pub fn load_config() -> Result<Config> {
    let proj_dirs = ProjectDirs::from("org", "runner", "runner");
    let config_path = if let Some(dirs) = &proj_dirs {
        dirs.config_dir().join("config.toml")
    } else {
        PathBuf::from("config.toml")
    };

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
