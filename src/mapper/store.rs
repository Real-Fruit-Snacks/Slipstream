use std::path::Path;
use serde::{Deserialize, Serialize};

use crate::mapper::parser::{ParsedEntry, ParsedUser};
use crate::logging::writer::atomic_write;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MapStore {
    pub entries: Vec<ParsedEntry>,
    pub users: Vec<ParsedUser>,
}

impl MapStore {
    pub fn new_empty() -> Self {
        Self::default()
    }

    pub fn load_or_create(path: &Path) -> Self {
        if path.exists() {
            if let Ok(data) = std::fs::read(path) {
                if let Ok(store) = serde_json::from_slice::<MapStore>(&data) {
                    return store;
                }
            }
        }
        Self::new_empty()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let data = serde_json::to_vec_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        atomic_write(path, &data)
    }

    pub fn add_entry(&mut self, entry: ParsedEntry) {
        if let Some(existing) = self.entries.iter_mut().find(|e| e.path == entry.path) {
            if entry.permissions.is_some() {
                existing.permissions = entry.permissions;
            }
            if entry.owner.is_some() {
                existing.owner = entry.owner;
            }
            if entry.size.is_some() {
                existing.size = entry.size;
            }
            if entry.entry_type != crate::mapper::parser::EntryType::Unknown {
                existing.entry_type = entry.entry_type;
            }
        } else {
            self.entries.push(entry);
        }
    }

    pub fn add_entries(&mut self, entries: Vec<ParsedEntry>) {
        for entry in entries {
            self.add_entry(entry);
        }
    }

    pub fn add_users(&mut self, users: Vec<ParsedUser>) {
        for user in users {
            if !self.users.iter().any(|u| u.username == user.username) {
                self.users.push(user);
            }
        }
    }

    pub fn entries(&self) -> &[ParsedEntry] {
        &self.entries
    }

    pub fn users(&self) -> &[ParsedUser] {
        &self.users
    }

    pub fn reset(&mut self) {
        self.entries.clear();
        self.users.clear();
    }
}
