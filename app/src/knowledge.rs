//! Knowledge-base loading: reads the JSON graph files (nodes, edges, paths,
//! questions, solutions, sessions, weak memory) from a `knowledge/` directory
//! into the in-memory [`KnowledgeBase`]. Missing files are empty collections.

use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{BreakingQuestion, ContextPath, Edge, Node, Session, Solution, WeakMemoryEntry};

/// The full in-memory knowledge base, loaded from the `knowledge/` directory.
#[derive(Debug, Default)]
#[allow(dead_code)] // fields are stubs for future phases
pub struct KnowledgeBase {
    /// All graph nodes.
    pub nodes: Vec<Node>,
    /// All graph edges (weighted transitions between nodes).
    pub edges: Vec<Edge>,
    /// Named, tagged reasoning paths (Phase 5+).
    #[allow(dead_code)]
    pub paths: Vec<ContextPath>,
    /// Breaking-question definitions (Phase 3+).
    #[allow(dead_code)]
    pub questions: Vec<BreakingQuestion>,
    /// Solution texts keyed by node id.
    #[allow(dead_code)]
    pub solutions: Vec<Solution>,
    /// Weak-memory entries (uncertain / rejected answers, Phase 9+).
    pub weak_memory: Vec<WeakMemoryEntry>,
    /// Recorded sessions with outcomes (Phase 7+).
    pub sessions: Vec<Session>,
}

impl KnowledgeBase {
    /// Load all JSON files from `dir`. Missing files are treated as empty
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

    /// Number of loaded nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of loaded edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read one JSON file from `dir` into a `Vec<T>`. A missing, empty, or `null`
/// file yields an empty vector; any other parse failure is returned as an error.
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
