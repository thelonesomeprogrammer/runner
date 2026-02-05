use crate::model::{Entry, EntryType};
use crate::sources::Source;
use anyhow::Result;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use log::{info, debug};

pub struct BinSource;

impl Source for BinSource {
    fn scan(&self) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();
        if let Ok(path_var) = env::var("PATH") {
            for path_str in path_var.split(':') {
                let path = std::path::Path::new(path_str);
                if path.exists() {
                    debug!("Scanning binaries in {:?}", path);
                    if let Ok(read_dir) = fs::read_dir(path) {
                        for entry in read_dir.flatten() {
                            let path = entry.path();
                            if path.is_file() {
                                if let Ok(metadata) = fs::metadata(&path) {
                                    // Check if executable
                                    if metadata.permissions().mode() & 0o111 != 0 {
                                         if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                                             entries.push(Entry::new(
                                                 path.to_string_lossy().to_string(),
                                                 file_name.to_string(),
                                                 path.to_string_lossy().to_string(),
                                                 EntryType::Binary,
                                                 false,
                                             ));
                                         }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        info!("BinSource: found {} entries", entries.len());
        Ok(entries)
    }
}
