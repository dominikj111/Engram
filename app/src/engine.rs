//! The reasoning engine: tokenise a query, seed node activation from label/tag
//! matches, propagate activation across the graph with decay, rank solution
//! nodes, and classify the outcome into a confidence level.
//!
//! Determinism is a hard invariant: activation state lives in `BTreeMap`s so
//! iteration and f32 summation order are identical for identical inputs.

use crate::knowledge::KnowledgeBase;
pub use crate::model::ConfidenceLevel;
use crate::model::{Node, NodeKind, Solution};

/// One step of the activation trace: the activation a node received through a
/// single edge during one propagation hop.
pub struct TraceStep {
    /// Propagation hop number (1-based).
    pub hop: u8,
    /// Activation of the edge's source node at the start of this hop.
    pub src_activation: f32,
    /// Label of the destination node that received activation.
    pub dst_label: String,
    /// Activation added to the destination node by this single edge.
    pub dst_activation: f32,
    /// Kind of the destination node.
    pub dst_kind: NodeKind,
    /// Edge weight applied to the propagated activation.
    pub edge_weight: f32,
}

/// The outcome of a query: the ranked top solution (if any), the confidence
/// classification, the activation seeds, and the full hop-by-hop trace.
pub struct QueryResult {
    /// Best solution as `(score, node, solution)`, `None` if nothing activated.
    pub top_solution: Option<(f32, Node, Solution)>,
    /// Confidence classification derived from θ_a and θ_d.
    pub confidence: ConfidenceLevel,
    /// Seeded nodes as `(label, activation, kind)` — populated when `explain`.
    pub seeds: Vec<(String, f32, NodeKind)>,
    /// Hop-by-hop propagation steps — populated when `explain`.
    pub trace: Vec<TraceStep>,
}

/// The query engine: pure computation over an immutable [`KnowledgeBase`].
pub struct Engine<'a> {
    kb: &'a KnowledgeBase,
}

impl<'a> Engine<'a> {
    /// Create an engine over the given knowledge base.
    pub fn new(kb: &'a KnowledgeBase) -> Self {
        Self { kb }
    }

    /// Run one query: seed → propagate (λ decay, max 4 hops) → rank → classify.
    ///
    /// Returns a [`QueryResult`] with the top solution, confidence level, and
    /// (when `explain` is true) the activation seeds and hop-by-hop trace.
    pub fn query(&self, query: &str, explain: bool) -> QueryResult {
        let tokens = self.tokenise(query);
        // BTreeMap, not HashMap: deterministic iteration order is a hard invariant
        // (same input must always produce the same output). HashMap uses RandomState
        // (per-process seed) and f32 summation is order-sensitive.
        let mut node_activations: std::collections::BTreeMap<u32, f32> =
            std::collections::BTreeMap::new();
        let mut trace = Vec::new();

        let node_map: std::collections::HashMap<u32, &Node> =
            self.kb.nodes.iter().map(|n| (n.id, n)).collect();

        // 1. Initial activation (Seeding)
        let mut seeds = Vec::new();
        for node in &self.kb.nodes {
            let mut best_score = 0.0;

            for token in &tokens {
                let score = if node.label == *token {
                    0.90
                } else if node.label.split('_').any(|part| part == *token) {
                    0.70
                } else if node.tags.contains(token) {
                    0.45
                } else {
                    0.0
                };

                if score > best_score {
                    best_score = score;
                }
            }

            if best_score > 0.0 {
                node_activations.insert(node.id, best_score);
                if explain {
                    seeds.push((node.label.clone(), best_score, node.kind.clone()));
                }
            }
        }

        // 2. Propagation
        let lambda = 0.85;
        let max_hops = 4;
        let mut current_activations = node_activations.clone();

        for hop in 1..=max_hops {
            let mut next_activations = std::collections::BTreeMap::new();
            let mut hop_fired = false;

            for edge in &self.kb.edges {
                if let Some(&src_activation) = current_activations.get(&edge.src) {
                    let added_activation = src_activation * edge.weight * lambda;
                    if added_activation > 0.0 {
                        *next_activations.entry(edge.dst).or_insert(0.0) += added_activation;
                        hop_fired = true;

                        if explain {
                            let dst_node = node_map.get(&edge.dst).unwrap();
                            trace.push(TraceStep {
                                hop: hop as u8,
                                src_activation,
                                dst_label: dst_node.label.clone(),
                                dst_activation: added_activation,
                                dst_kind: dst_node.kind.clone(),
                                edge_weight: edge.weight,
                            });
                        }
                    }
                }
            }

            if !hop_fired {
                break;
            }

            // Accumulate into main activations
            for (node_id, activation) in next_activations.iter() {
                *node_activations.entry(*node_id).or_insert(0.0) += activation;
            }
            current_activations = next_activations;
        }

        // 3. Ranking and Confidence (to be refined in next step, but fulfilling QueryResult now)
        let mut solutions: Vec<(f32, Node, Solution)> = node_activations
            .iter()
            .filter_map(|(node_id, &score)| {
                let node = self
                    .kb
                    .nodes
                    .iter()
                    .find(|n| n.id == *node_id && n.kind == NodeKind::Solution)?;
                let solution = self.kb.solutions.iter().find(|s| s.node_id == *node_id)?;
                Some((score, node.clone(), solution.clone()))
            })
            .collect();

        solutions.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                // Deterministic tie-break: equal scores resolve by node id (desc).
                .then_with(|| b.1.id.cmp(&a.1.id))
        });

        let theta_a = 0.75;
        let theta_d = 0.15;

        let confidence = if solutions.is_empty() {
            ConfidenceLevel::Unknown
        } else {
            let top_score = solutions[0].0;
            if top_score >= theta_a {
                if solutions.len() > 1 {
                    let second_score = solutions[1].0;
                    if top_score - second_score >= theta_d {
                        ConfidenceLevel::High
                    } else {
                        ConfidenceLevel::Medium
                    }
                } else {
                    ConfidenceLevel::High
                }
            } else {
                ConfidenceLevel::Low
            }
        };

        QueryResult {
            top_solution: solutions.first().cloned(),
            confidence,
            seeds,
            trace,
        }
    }

    /// Tokenise a query into lowercase keyword tokens: split on non-alphanumerics
    /// and drop stop words. Phase 1 simple tokeniser; superseded by BM25 in
    /// Phase 13.
    pub fn tokenise(&self, input: &str) -> Vec<String> {
        // Stop words to discard.
        const STOP: &[&str] = &[
            "a", "an", "the", "is", "it", "i", "my", "me", "we", "our", "you", "your", "do",
            "does", "did", "am", "are", "was", "were", "be", "been", "being", "get", "got", "have",
            "has", "had", "not", "no", "so", "to", "for", "of", "in", "on", "at", "by", "or",
            "and", "but", "if", "that", "this", "with", "from", "when", "why", "how", "what",
            "where", "can", "will", "would", "should", "keep", "getting", "keep", "always",
            "still", "just", "even", "only", "also",
        ];

        input
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty() && !STOP.contains(t))
            .map(|t| t.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge::KnowledgeBase;
    use std::path::Path;

    fn fixture() -> KnowledgeBase {
        KnowledgeBase::load(Path::new("knowledge")).expect("fixture knowledge loads")
    }

    #[test]
    fn propagation_reaches_solution_across_hops() {
        // network_issue (13) → timeout_context (14) → timeout (8) → fix_timeout (26)
        // is a 4-hop chain in the seed knowledge; activation must reach the solution.
        let kb = fixture();
        let engine = Engine::new(&kb);
        let res = engine.query("network", true);
        let (score, node, _) = res.top_solution.expect("network resolves to a solution");
        assert_eq!(node.label, "fix_timeout");
        assert!(score > 0.0, "solution must carry positive activation");
        assert!(
            !res.trace.is_empty(),
            "explain must produce a hop-by-hop trace"
        );
    }

    #[test]
    fn deterministic_tie_break_and_output() {
        // "error" seeds node `error`, which fans out to five solutions with identical
        // accumulated scores. The result must be stable: ties resolve by node id (desc),
        // and two runs over the same graph must agree bit-for-bit.
        let kb = fixture();
        let engine = Engine::new(&kb);
        let a = engine.query("error", true);
        let b = engine.query("error", true);

        let top_a = a.top_solution.expect("error resolves");
        let top_b = b.top_solution.expect("error resolves");
        assert_eq!(
            top_a.0.to_bits(),
            top_b.0.to_bits(),
            "scores must be bit-identical"
        );
        assert_eq!(top_a.1.id, top_b.1.id, "tie-break must be stable");
        assert_eq!(a.confidence, b.confidence);
        assert_eq!(a.trace.len(), b.trace.len());
        for (x, y) in a.trace.iter().zip(b.trace.iter()) {
            assert_eq!(x.hop, y.hop);
            assert_eq!(x.dst_label, y.dst_label);
            assert_eq!(x.dst_activation.to_bits(), y.dst_activation.to_bits());
        }
    }
}
