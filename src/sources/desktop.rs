use crate::model::{Entry, EntryType};
use crate::sources::Source;
use anyhow::Result;
use std::fs;
use std::path::Path;
use directories::BaseDirs;
use log::{info, debug};

pub struct DesktopSource;

impl Source for DesktopSource {
    fn scan(&self) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();
        let mut data_dirs = Vec::new();

        if let Some(base_dirs) = BaseDirs::new() {
            data_dirs.push(base_dirs.data_dir().join("applications"));
        }
        data_dirs.push(Path::new("/usr/share/applications").to_path_buf());
        data_dirs.push(Path::new("/usr/local/share/applications").to_path_buf());

        for dir in data_dirs {
            if dir.exists() {
                debug!("Scanning desktop files in {:?}", dir);
                if let Ok(read_dir) = fs::read_dir(dir) {
                    for entry in read_dir.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                            if let Ok(content) = fs::read_to_string(&path) {
                                if let Some((name, exec, term, icon, container)) = parse_desktop_file(&content) {
                                     let display_name = if let Some(c) = &container {
                                         format!("{} ({})", name, c)
                                     } else {
                                         name
                                     };
                                     
                                     let mut entry = Entry::new(
                                         path.to_string_lossy().to_string(),
                                         display_name,
                                         exec,
                                         EntryType::Desktop,
                                         term,
                                     );
                                     entry.icon = icon;
                                     entry.is_container = container.is_some();
                                     entries.push(entry);
                                }
                            }
                        }
                    }
                }
            }
        }
        info!("DesktopSource: found {} entries", entries.len());
        Ok(entries)
    }
}

fn parse_desktop_file(content: &str) -> Option<(String, String, bool, Option<String>, Option<String>)> {
    let mut name = None;
    let mut exec = None;
    let mut terminal = false;
    let mut no_display = false;
    let mut icon = None;
    let mut is_desktop_entry = false;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') { continue; }

        if line == "[Desktop Entry]" {
            is_desktop_entry = true;
            continue;
        }
        
        if line.starts_with('[') {
            is_desktop_entry = false; 
            continue;
        }

        if !is_desktop_entry { continue; }

        if line.starts_with("Name=") {
            name = Some(line.trim_start_matches("Name=").to_string());
        } else if line.starts_with("Exec=") {
            let raw_exec = line.trim_start_matches("Exec=");
            let clean_exec: String = raw_exec.split_whitespace()
                .filter(|s| !s.starts_with('%'))
                .collect::<Vec<_>>()
                .join(" ");
            exec = Some(clean_exec);
        } else if line.starts_with("Terminal=") {
            terminal = line.trim_start_matches("Terminal=") == "true";
        } else if line.starts_with("NoDisplay=") {
            no_display = line.trim_start_matches("NoDisplay=") == "true";
        } else if line.starts_with("Icon=") {
            icon = Some(line.trim_start_matches("Icon=").to_string());
        }
    }

    if no_display { return None; }

    let mut container = None;
    if let Some(cmd) = &exec {
        if cmd.contains("distrobox-enter") {
            // naive parse: -n <name> or -n <name>
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            if let Some(pos) = parts.iter().position(|&x| x == "-n" || x == "--name") {
                if pos + 1 < parts.len() {
                    container = Some(parts[pos+1].to_string());
                }
            }
        } else if cmd.contains("toolbox run") {
             let parts: Vec<&str> = cmd.split_whitespace().collect();
             if let Some(pos) = parts.iter().position(|&x| x == "-c" || x == "--container") {
                if pos + 1 < parts.len() {
                    container = Some(parts[pos+1].to_string());
                }
            }
        }
    }

    match (name, exec) {
        (Some(n), Some(e)) => Some((n, e, terminal, icon, container)),
        _ => None,
    }
}
