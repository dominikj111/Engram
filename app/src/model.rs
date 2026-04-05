use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum NodeKind {
    Concept,
    Question,
    Solution,
    Latent,
}

impl std::fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeKind::Concept => write!(f, "concept"),
            NodeKind::Question => write!(f, "question"),
            NodeKind::Solution => write!(f, "solution"),
            NodeKind::Latent => write!(f, "latent"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: u32,
    pub label: String,
    pub kind: NodeKind,
    /// Transient activation score during query processing (not persisted).
    #[serde(default)]
    pub activation: f32,
    #[serde(default)]
    pub tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// Edge
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub src: u32,
    pub dst: u32,
    #[serde(default = "default_half")]
    pub weight: f32,
    #[serde(default = "default_half")]
    pub confidence: f32,
    #[serde(default)]
    pub usage_count: u32,
    #[serde(default)]
    pub path_labels: Vec<String>,
}

fn default_half() -> f32 {
    0.5
}

// ---------------------------------------------------------------------------
// ContextPath
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPath {
    pub id: u32,
    pub name: String,
    #[serde(default)]
    pub node_ids: Vec<u32>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub usage_count: u32,
    #[serde(default)]
    pub avg_confidence: f32,
}

// ---------------------------------------------------------------------------
// Breaking questions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub answer_token: String,
    pub target_node: u32,
    pub path_label: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingQuestion {
    pub id: u32,
    pub label: String,
    pub prompt: String,
    #[serde(default)]
    pub branches: Vec<Branch>,
}

// ---------------------------------------------------------------------------
// Solution text
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution {
    pub node_id: u32,
    pub text: String,
}

// ---------------------------------------------------------------------------
// Weak memory
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeakMemoryStatus {
    Uncertain,
    Rejected,
    Resolved,
}

impl std::fmt::Display for WeakMemoryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeakMemoryStatus::Uncertain => write!(f, "uncertain"),
            WeakMemoryStatus::Rejected => write!(f, "rejected"),
            WeakMemoryStatus::Resolved => write!(f, "resolved"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeakMemoryEntry {
    pub id: String,
    pub question: String,
    #[serde(default)]
    pub tokens: Vec<String>,
    pub attempted_path: String,
    pub attempted_solution: String,
    pub status: WeakMemoryStatus,
    pub session_id: String,
    pub correction: Option<String>,
}

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionOutcome {
    Confirmed,
    Rejected,
    Abandoned,
}

impl std::fmt::Display for SessionOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionOutcome::Confirmed => write!(f, "confirmed"),
            SessionOutcome::Rejected => write!(f, "rejected"),
            SessionOutcome::Abandoned => write!(f, "abandoned"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    #[serde(default)]
    pub path_labels: Vec<String>,
    #[serde(default)]
    pub breaking_questions_asked: Vec<String>,
    pub outcome: SessionOutcome,
}
