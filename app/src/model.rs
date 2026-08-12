//! Domain model: the JSON-serialisable data structures of the knowledge graph
//! — nodes, edges, context paths, breaking questions, solutions, sessions, and
//! weak memory. Everything the engine reads from `knowledge/` lives here.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Node
// ---------------------------------------------------------------------------

/// The kind of a graph node — determines its role in reasoning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum NodeKind {
    /// A topic/concept node that activation flows through.
    Concept,
    /// A breaking-question node that disambiguates candidate solutions.
    Question,
    /// A terminal node carrying an answer text.
    Solution,
    /// An automatically discovered node for hidden shared concepts.
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

/// A graph node: a concept, question, solution, or latent node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Stable numeric identifier.
    pub id: u32,
    /// Human-readable label (snake_case), e.g. `borrow_checker`.
    pub label: String,
    /// Role of the node in reasoning.
    pub kind: NodeKind,
    /// Transient activation score during query processing (not persisted).
    #[serde(default)]
    pub activation: f32,
    /// Semantic tags across the domain/pattern/scope taxonomy.
    #[serde(default)]
    pub tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// Edge
// ---------------------------------------------------------------------------

/// A weighted, confidence-bearing transition between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Source node id.
    pub src: u32,
    /// Destination node id.
    pub dst: u32,
    /// Transition weight used in activation propagation.
    #[serde(default = "default_half")]
    pub weight: f32,
    /// Confidence in the transition; reinforced/decayed by learning.
    #[serde(default = "default_half")]
    pub confidence: f32,
    /// Number of sessions that traversed this edge.
    #[serde(default)]
    pub usage_count: u32,
    /// Names of the context paths this edge participates in.
    #[serde(default)]
    pub path_labels: Vec<String>,
}

/// Default edge weight/confidence when a field is absent in JSON.
fn default_half() -> f32 {
    0.5
}

// ---------------------------------------------------------------------------
// ContextPath
// ---------------------------------------------------------------------------

/// A named, tagged reasoning route recorded after a confirmed traversal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPath {
    /// Stable numeric identifier.
    pub id: u32,
    /// Human-readable name, e.g. `rust_ownership_violation`.
    pub name: String,
    /// Ordered node sequence of the path.
    #[serde(default)]
    pub node_ids: Vec<u32>,
    /// Semantic tags of the path.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Number of times the path has been traversed.
    #[serde(default)]
    pub usage_count: u32,
    /// Average confidence of the traversed edges.
    #[serde(default)]
    pub avg_confidence: f32,
}

// ---------------------------------------------------------------------------
// Breaking questions
// ---------------------------------------------------------------------------

/// One possible answer of a breaking question, committing to a context path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    /// Expected user token ("yes", "no", "multiple", …).
    pub answer_token: String,
    /// Node to activate when this branch is chosen.
    pub target_node: u32,
    /// Path label committed to the session on this branch.
    pub path_label: String,
    /// Tags associated with this branch.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A breaking question: a labelled decomposition with mutually exclusive
/// branches that partition the candidate solution space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingQuestion {
    /// Stable numeric identifier.
    pub id: u32,
    /// Domain label, e.g. `ownership_dimension`.
    pub label: String,
    /// Text shown to the user.
    pub prompt: String,
    /// Mutually exclusive answer branches.
    #[serde(default)]
    pub branches: Vec<Branch>,
}

// ---------------------------------------------------------------------------
// Solution text
// ---------------------------------------------------------------------------

/// The answer text attached to a solution node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution {
    /// The solution node this text belongs to.
    pub node_id: u32,
    /// The answer text.
    pub text: String,
}

// ---------------------------------------------------------------------------
// Weak memory
// ---------------------------------------------------------------------------

/// Lifecycle status of a weak-memory entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeakMemoryStatus {
    /// Attempted but not yet confirmed or rejected.
    Uncertain,
    /// Attempted and rejected by the user.
    Rejected,
    /// Corrected and promoted back into the graph.
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

/// A record of an uncertain or failed reasoning attempt, stored for later
/// correction. Never contains user-originated text — only activation patterns
/// and curated path names.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeakMemoryEntry {
    /// Stable entry id (e.g. `wm-0042`).
    pub id: String,
    /// Node IDs activated when this query was processed.
    /// Input text is never stored — the activation pattern is sufficient
    /// to diagnose the failure and connect the correct solution node.
    #[serde(default)]
    pub activated_nodes: Vec<u32>,
    /// Path label of the attempted (and failed) reasoning route. Curated name, not user text.
    pub attempted_path: String,
    /// Solution node that was proposed but rejected. None if session was abandoned.
    pub attempted_solution_node: Option<u32>,
    /// Current lifecycle status.
    pub status: WeakMemoryStatus,
    /// Session that produced this entry.
    pub session_id: String,
    /// Correct solution node, populated when a correction is confirmed.
    pub correction_node: Option<u32>,
}

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

/// Outcome of a recorded session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionOutcome {
    /// The proposed solution was accepted.
    Confirmed,
    /// The proposed solution was rejected.
    Rejected,
    /// The session ended without resolution.
    Abandoned,
}

/// Confidence classification of a query result, derived from θ_a and θ_d.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    /// Top score ≥ θ_a AND gap to second ≥ θ_d.
    High,
    /// Top score ≥ θ_a BUT gap to second < θ_d.
    Medium,
    /// Top score < θ_a.
    Low,
    /// No candidates reached activation at all.
    Unknown,
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

/// A recorded user session with its outcome and the questions asked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Stable, timestamped session id (e.g. `2026-03-06-001`).
    pub session_id: String,
    /// Curated path labels traversed during this session (e.g. "rust_ownership_violation").
    #[serde(default)]
    pub path_labels: Vec<String>,
    /// IDs of breaking question nodes asked during this session.
    /// Stores node IDs, not question text or user responses.
    #[serde(default)]
    pub breaking_questions_asked: Vec<u32>,
    /// How the session ended.
    pub outcome: SessionOutcome,
}
