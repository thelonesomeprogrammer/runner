#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryType {
    Desktop,
    Binary,
    History,
    Custom,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Entry {
    pub id: String,            // Unique ID (e.g., "firefox.desktop" or "ls")
    pub name: String,          // Display name
    pub command: String,       // Executable command
    pub icon: Option<String>,  // Icon name/path
    pub score: i64,            // Fuzzy match score
    pub group: String,         // The launch group it belongs to
    pub is_container: bool,    // Context hint
    pub open_in_terminal: bool,
    pub entry_type: EntryType,
}

impl Entry {
    pub fn new(id: String, name: String, command: String, entry_type: EntryType, open_in_terminal: bool) -> Self {
        Self {
            id,
            name,
            command,
            icon: None,
            score: 0,
            group: "default".to_string(),
            is_container: false,
            open_in_terminal,
            entry_type,
        }
    }
}
