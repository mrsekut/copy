use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct History {
    pub entries: Vec<HistoryEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub repo: String,
    pub file_path: String,
}

impl History {
    pub fn load() -> io::Result<Self> {
        let history_path = get_history_path()?;

        match fs::read_to_string(&history_path) {
            Ok(content) => serde_json::from_str(&content).map_err(|e| {
                io::Error::new(
                    ErrorKind::InvalidData,
                    format!("Invalid history file: {}", e),
                )
            }),
            Err(e) if e.kind() == ErrorKind::NotFound => {
                // Create new history if history file does not exist
                Ok(History {
                    entries: Vec::new(),
                })
            }
            Err(e) => Err(e),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let history_path = get_history_path()?;

        // Create directory if it doesn't exist
        if let Some(parent) = history_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;

        fs::write(history_path, content)
    }

    pub fn add_entry(&mut self, repo: &str, file_path: &str) -> io::Result<()> {
        // Remove existing entry if already present (to add at the beginning later)
        self.entries
            .retain(|entry| !(entry.repo == repo && entry.file_path == file_path));

        // Add new entry at the beginning
        self.entries.insert(
            0,
            HistoryEntry {
                repo: repo.to_string(),
                file_path: file_path.to_string(),
            },
        );

        // Limit history to a maximum number (e.g., latest 50 items)
        if self.entries.len() > 50 {
            self.entries.truncate(50);
        }

        self.save()
    }

    pub fn get_history_items(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|entry| format!("{}: {}", entry.repo, entry.file_path))
            .collect()
    }
}

fn get_history_path() -> io::Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "Home directory not found"))?;

    Ok(home.join(".config").join("copy").join("history.json"))
}
