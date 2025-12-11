use rusqlite::{params, Connection};
use bincode;

pub struct VectorDB {
    conn: Connection,
}

impl VectorDB {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS embeddings (
                path TEXT PRIMARY KEY,
                vector BLOB,
                modified INTEGER
            );"
        )?;

        Ok(Self { conn })
    }

    pub fn store_embedding(&self, file_path: &str, vector: &[f32], modified: i64) -> anyhow::Result<()> {
        let blob = bincode::serialize(vector)?;

        self.conn.execute(
            "INSERT OR REPLACE INTO embeddings (path, vector, modified)
             VALUES (?1, ?2, ?3)",
            params![file_path, blob, modified],
        )?;
        Ok(())
    }

    pub fn load_all(&self) -> anyhow::Result<Vec<(String, Vec<f32>)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT path, vector FROM embeddings")?;

        let rows = stmt.query_map([], |row| {
            let path: String = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            let vector: Vec<f32> = bincode::deserialize(&blob).unwrap();
            Ok((path, vector))
        })?;

        let mut results = Vec::new();
        for r in rows {
            results.push(r?);
        }
        Ok(results)
    }
}
