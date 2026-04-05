use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{
    BreakingQuestion, ContextPath, Edge, Node, Session, Solution, WeakMemoryEntry,
};

/// The full in-memory knowledge base, loaded from the `knowledge/` directory.
#[derive(Debug, Default)]
#[allow(dead_code)] // fields are stubs for future phases
pub struct KnowledgeBase {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    #[allow(dead_code)]
    pub paths: Vec<ContextPath>,
    #[allow(dead_code)]
    pub questions: Vec<BreakingQuestion>,
    #[allow(dead_code)]
    pub solutions: Vec<Solution>,
    pub weak_memory: Vec<WeakMemoryEntry>,
    pub sessions: Vec<Session>,
}

impl KnowledgeBase {
    /// Load all JSON files from `dir`.  Missing files are treated as empty
    /// collections; parse errors are surfaced immediately.
    pub fn load(dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            nodes: load_json(dir, "nodes.json")?,
            edges: load_json(dir, "edges.json")?,
            paths: load_json(dir, "paths.json")?,
            questions: load_json(dir, "questions.json")?,
            solutions: load_json(dir, "solutions.json")?,
            weak_memory: load_json(dir, "weak_memory.json")?,
            sessions: load_json(dir, "sessions.json")?,
        })
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn load_json<T>(dir: &Path, filename: &str) -> Result<Vec<T>, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let path: PathBuf = dir.join(filename);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents = fs::read_to_string(&path)?;
    let trimmed = contents.trim();
    if trimmed.is_empty() || trimmed == "null" {
        return Ok(Vec::new());
    }
    let items: Vec<T> = serde_json::from_str(trimmed)?;
    Ok(items)
}
