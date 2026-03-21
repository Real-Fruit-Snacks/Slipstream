use crate::mapper::parser::ParsedEntry;
use crate::mapper::store::MapStore;

fn split_parent_name(path: &str) -> Option<(&str, &str)> {
    path.rsplit_once('/').or_else(|| path.rsplit_once('\\'))
}

fn path_depth(path: &str) -> usize {
    path.chars().filter(|c| *c == '/' || *c == '\\').count()
}

pub struct MapQuery;

impl MapQuery {
    /// Returns direct children of `dir` only (not recursive).
    pub fn list_directory<'a>(store: &'a MapStore, dir: &str) -> Vec<&'a ParsedEntry> {
        let dir = dir.trim_end_matches('/').trim_end_matches('\\');
        let dir_lower = dir.to_lowercase();
        store
            .entries()
            .iter()
            .filter(|e| {
                if let Some((parent, _)) = split_parent_name(&e.path) {
                    let parent_norm = parent.trim_end_matches('/').trim_end_matches('\\');
                    parent_norm == dir || parent_norm.to_lowercase() == dir_lower
                } else {
                    false
                }
            })
            .collect()
    }

    /// Find entries by pattern:
    /// - "suid" returns SUID entries
    /// - "*.ext" glob pattern matches by name suffix
    /// - otherwise substring match on path
    pub fn find<'a>(store: &'a MapStore, pattern: &str) -> Vec<&'a ParsedEntry> {
        if pattern == "suid" {
            return store.entries().iter().filter(|e| e.is_suid()).collect();
        }

        if let Some(suffix) = pattern.strip_prefix("*.") {
            return store
                .entries()
                .iter()
                .filter(|e| e.name.ends_with(&format!(".{}", suffix)))
                .collect();
        }

        store
            .entries()
            .iter()
            .filter(|e| e.path.contains(pattern))
            .collect()
    }

    /// Show number of entries and list directories with entry counts.
    pub fn coverage(store: &MapStore) -> String {
        use std::collections::HashMap;
        let entries = store.entries();
        let total = entries.len();

        let mut dir_counts: HashMap<String, usize> = HashMap::new();
        for entry in entries {
            if let Some((parent, _)) = split_parent_name(&entry.path) {
                let parent = if parent.is_empty() { "/" } else { parent };
                *dir_counts.entry(parent.to_string()).or_insert(0) += 1;
            }
        }

        let mut lines = vec![format!("{} entries", total)];
        let mut dirs: Vec<String> = dir_counts.keys().cloned().collect();
        dirs.sort();
        for dir in dirs {
            lines.push(format!("{}: {} entries", dir, dir_counts[&dir]));
        }
        lines.join("\n")
    }

    /// Serialize all entries to pretty-printed JSON.
    pub fn export_json(store: &MapStore) -> String {
        serde_json::to_string_pretty(store.entries()).unwrap_or_else(|_| "[]".to_string())
    }

    /// Format all entries as an indented tree sorted by path.
    pub fn format_tree(store: &MapStore) -> String {
        let mut paths: Vec<&str> = store.entries().iter().map(|e| e.path.as_str()).collect();
        paths.sort();

        let mut lines = Vec::new();
        for path in paths {
            let depth = path_depth(path).saturating_sub(1);
            let name = split_parent_name(path).map(|(_, n)| n).unwrap_or(path);
            let indent = "  ".repeat(depth);
            lines.push(format!("{}{}", indent, name));
        }

        lines.join("\n")
    }
}
