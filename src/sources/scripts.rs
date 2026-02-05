use crate::model::{Entry, EntryType};
use crate::sources::Source;
use anyhow::Result;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use log::{info, debug};
use directories::ProjectDirs;

pub struct ScriptsSource;

impl Source for ScriptsSource {
    fn scan(&self) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();
        
        let proj_dirs = ProjectDirs::from("org", "runner", "runner");
        let scripts_dir = if let Some(dirs) = proj_dirs {
            dirs.config_dir().join("scripts")
        } else {
            return Ok(vec![]);
        };

        if !scripts_dir.exists() {
            debug!("Scripts directory {:?} does not exist, skipping", scripts_dir);
            return Ok(vec![]);
        }

        debug!("Scanning scripts in {:?}", scripts_dir);
        if let Ok(read_dir) = fs::read_dir(scripts_dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = fs::metadata(&path) {
                        // Check if executable
                        if metadata.permissions().mode() & 0o111 != 0 {
                             if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                                 // Scripts usually don't have icons, but we could look for name.png
                                 entries.push(Entry::new(
                                     path.to_string_lossy().to_string(),
                                     file_name.to_string(),
                                     path.to_string_lossy().to_string(),
                                     EntryType::Custom,
                                     false, // Assume scripts manage their own terminal state or don't need one
                                 ));
                             }
                        }
                    }
                }
            }
        }
        
        info!("ScriptsSource: found {} entries", entries.len());
        Ok(entries)
    }
}
