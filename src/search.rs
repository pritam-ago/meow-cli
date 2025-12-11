use std::fs;
use std::path::{Path, PathBuf};

/// Recursively search filesystem for file/folder names that contain the query.
/// Returns a list of matching paths.
pub fn search_files(root: &Path, query: &str) -> Vec<PathBuf> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    fn walk(dir: &Path, query: &str, results: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                if name.contains(query) {
                    results.push(path.clone());
                }

                // If directory, recurse
                if path.is_dir() {
                    walk(&path, query, results);
                }
            }
        }
    }

    walk(root, &query_lower, &mut results);
    results
}
