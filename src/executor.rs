use std::process::{Command, Stdio};
use anyhow::Result;
use crate::model::Entry;
use crate::config::Config;
use crate::sources::history;

pub fn execute(entry: &Entry, config: &Config, active_group: &str) -> Result<()> {
    // Increment usage history
    let _ = history::increment_usage(&entry.id);

    // Basic execution logic with Terminal support
    
    let mut cmd_parts = Vec::new();
    
    if entry.open_in_terminal {
        if let Some(term_cmd) = &config.general.terminal {
             cmd_parts.extend(term_cmd.split_whitespace());
             cmd_parts.push(&entry.command);
        } else {
             cmd_parts.extend(entry.command.split_whitespace());
        }
    } else {
         cmd_parts.extend(entry.command.split_whitespace());
    }

    if cmd_parts.is_empty() {
        return Ok(());
    }

    let mut command = Command::new(cmd_parts[0]);
    command.args(&cmd_parts[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // Apply group env overrides
    if let Some(group_config) = config.groups.get(active_group) {
        if let Some(env) = &group_config.env {
            for (key, val) in env {
                command.env(key, val);
            }
        }
    }
    
    command.spawn()?;
    
    Ok(())
}
