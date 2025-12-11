use crate::ai::AiAction;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::{Duration, Local};

pub fn execute_action(action: AiAction) {
    match action.intent.as_str() {
        "search" => execute_search(action),
        "open" => println!("(open not implemented yet)"),
        "read" => println!("(read not implemented yet)"),
        "summarize" => println!("(summarize not implemented yet)"),
        "delete" => println!("(delete not implemented yet)"),
        _ => println!("❌ Unknown intent: {}", action.intent),
    }
}

fn execute_search(action: AiAction) {
    println!("🔍 Executing AI search...");
    println!("→ Query: {:?}", action.query);
    println!("→ File type: {:?}", action.file_type);
    println!("→ Time: {:?}", action.time_filter);
    println!("→ Folder hint: {:?}", action.folder_hint);

    // 1. Pick search root
    let root = resolve_folder_hint(&action.folder_hint, &action.query);

    println!("📁 Searching in: {}", root.display());

    // 2. Walk the filesystem
    let mut matches = Vec::new();
    walk(&root, &mut matches);

    // 3. Filter by file type
    if let Some(ftype) = action.file_type {
        let ftype = ftype.to_lowercase();

        matches.retain(|p| {
            p.extension()
                .and_then(|e| e.to_str())                 // convert OsStr → &str
                .map(|ext| ext.to_lowercase() == ftype)  // compare string to string
                .unwrap_or(false)
    });
}


    // 4. Filter by time window
    if let Some(time) = action.time_filter {
        matches.retain(|path| file_matches_time(path, &time));
    }

    // 5. Basic keyword search (semantic comes next)
    if let Some(query) = action.query {
        let q = query.to_lowercase();
        matches.retain(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_lowercase().contains(&q))
                .unwrap_or(false)
        });
    }

    // Final output
    if matches.is_empty() {
        println!("😿 No matches found.");
    } else {
        println!("😼 Found {} result(s):", matches.len());
        for m in matches {
            println!("• {}", m.display());
        }
    }
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                out.push(path.clone());
            } else if path.is_dir() {
                walk(&path, out);
            }
        }
    }
}

fn resolve_folder_hint(hint: &Option<String>, query: &Option<String>) -> PathBuf {
    let home = dirs::home_dir().unwrap_or(PathBuf::from("."));

    // 1️⃣ If AI explicitly suggests a folder
    if let Some(h) = hint {
        return match h.to_lowercase().as_str() {
            "downloads" => home.join("Downloads"),
            "documents" => home.join("Documents"),
            "desktop" => home.join("Desktop"),
            _ => PathBuf::from("."),
        };
    }

    // 2️⃣ If user query mentions downloading
    if let Some(q) = query {
        let q = q.to_lowercase();
        if q.contains("download") || q.contains("downloaded") {
            return home.join("Downloads");
        }
    }

    // 3️⃣ Default fallback
    PathBuf::from(".")
}


// Basic time filter
fn file_matches_time(path: &PathBuf, filter: &str) -> bool {
    if let Ok(meta) = fs::metadata(path) {
        if let Ok(modified) = meta.modified() {
            let modified_dt = chrono::DateTime::<Local>::from(modified);

            let now = Local::now();

            match filter {
                "yesterday" => {
                    let yesterday = now - Duration::days(1);
                    modified_dt.date_naive() == yesterday.date_naive()
                }
                "today" => modified_dt.date_naive() == now.date_naive(),
                _ => true, // no filter
            }
        } else {
            false
        }
    } else {
        false
    }
}
