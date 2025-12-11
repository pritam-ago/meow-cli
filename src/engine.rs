use crate::ai::AiAction;
use crate::embedding::embed_text;
use crate::similarity::cosine_similarity;
use crate::vector_db::VectorDB;
use chrono::{Duration, Local};
use std::fs;
use std::path::{Path, PathBuf};

/// Top-level execution engine
pub fn execute_action(action: AiAction) -> anyhow::Result<()> {
    match action.intent.as_str() {
        "search" => {
            execute_search(action)?;
            Ok(())
        }
        "open" => {
            println!("(open not implemented yet)");
            Ok(())
        }
        "read" => {
            println!("(read not implemented yet)");
            Ok(())
        }
        "summarize" => {
            println!("(summarize not implemented yet)");
            Ok(())
        }
        "delete" => {
            println!("(delete not implemented yet)");
            Ok(())
        }
        _ => {
            println!("❌ Unknown intent: {}", action.intent);
            Ok(())
        }
    }
}

/// Search implementation (literal + semantic)
fn execute_search(action: AiAction) -> anyhow::Result<()> {
    println!("🔍 Executing AI search...");
    println!("→ Query: {:?}", action.query);
    println!("→ File type: {:?}", action.file_type);
    println!("→ Time: {:?}", action.time_filter);
    println!("→ Folder hint: {:?}", action.folder_hint);

    // 1. Pick search root
    let root = resolve_folder_hint(&action.folder_hint, &action.query);
    println!("📁 Searching in: {}", root.display());

    // 2. Walk filesystem
    let mut matches = Vec::new();
    walk(&root, &mut matches);

    // 3. Filter by file type
    if let Some(ftype) = &action.file_type {
        let ftype = ftype.to_lowercase();
        matches.retain(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|ext| ext.to_lowercase() == ftype)
                .unwrap_or(false)
        });
    }

    // 4. Filter by time window
    if let Some(time) = &action.time_filter {
        matches.retain(|path| file_matches_time(path, time));
    }

    // 5. Basic keyword search
    if let Some(query) = &action.query {
        let q = query.to_lowercase();
        matches.retain(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_lowercase().contains(&q))
                .unwrap_or(false)
        });
    }

    // ---------------------------
    // SEMANTIC SEARCH BEGINS HERE
    // ---------------------------

    let db = VectorDB::new("meow_vectors.db")?;

    println!("🧠 Generating embedding for query...");
    let query_vec = embed_text(&action.query.clone().unwrap_or_default())?;

    println!("🔍 Loading file embeddings...");
    let file_vectors = db.load_all()?; // Vec<(path, vector)>

    // Compute cosine similarity for each file
    let mut scored: Vec<(String, f32)> = file_vectors
        .into_iter()
        .map(|(path, vec)| {
            let score = cosine_similarity(&query_vec, &vec);
            (path, score)
        })
        .collect();

    // Sort by descending score
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));

    println!("😼 Top semantic matches:");
    for (path, score) in scored.iter().take(10) {
        println!("{:.3} → {}", score, path);
    }

    // ---------------------------
    // LITERAL RESULTS
    // ---------------------------

    if matches.is_empty() {
        println!("😿 No literal matches.");
    } else {
        println!("\n📄 Literal filename matches:");
        for m in matches {
            println!("• {}", m.display());
        }
    }

    Ok(())
}

/// Recursively walk directory tree
fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                out.push(path);
            } else if path.is_dir() {
                walk(&path, out);
            }
        }
    }
}

/// Smart folder resolver
fn resolve_folder_hint(hint: &Option<String>, query: &Option<String>) -> PathBuf {
    let home = dirs::home_dir().unwrap_or(PathBuf::from("."));

    // AI-provided folder
    if let Some(h) = hint {
        return match h.to_lowercase().as_str() {
            "downloads" => home.join("Downloads"),
            "documents" => home.join("Documents"),
            "desktop" => home.join("Desktop"),
            _ => PathBuf::from("."),
        };
    }

    // Infer from user query
    if let Some(q) = query {
        let q = q.to_lowercase();
        if q.contains("download") || q.contains("downloaded") {
            return home.join("Downloads");
        }
    }

    // Otherwise search current directory
    PathBuf::from(".")
}

/// Time-based filtering (yesterday, today...)
fn file_matches_time(path: &PathBuf, filter: &str) -> bool {
    if let Ok(meta) = fs::metadata(path) {
        if let Ok(modified) = meta.modified() {
            let modified_dt = chrono::DateTime::<Local>::from(modified);
            let now = Local::now();

            match filter {
                "yesterday" => {
                    let yesterday = now - Duration::days(1);
                    return modified_dt.date_naive() == yesterday.date_naive();
                }
                "today" => {
                    return modified_dt.date_naive() == now.date_naive();
                }
                _ => return true,
            }
        }
    }
    false
}
