use crate::knowledge::KnowledgeBase;
pub use crate::model::ConfidenceLevel;
use crate::model::{Node, Solution, NodeKind};

pub struct TraceStep {
    pub hop: u8,
    pub src_activation: f32,
    pub dst_label: String,
    pub dst_activation: f32,
    pub dst_kind: NodeKind,
    pub edge_weight: f32,
}

pub struct QueryResult {
    pub top_solution: Option<(f32, Node, Solution)>,
    pub confidence: ConfidenceLevel,
    pub seeds: Vec<(String, f32, NodeKind)>,
    pub trace: Vec<TraceStep>,
}

pub struct Engine<'a> {
    kb: &'a KnowledgeBase,
}

impl<'a> Engine<'a> {
    pub fn new(kb: &'a KnowledgeBase) -> Self {
        Self { kb }
    }

    pub fn query(&self, query: &str, explain: bool) -> QueryResult {
        let tokens = self.tokenise(query);
        let mut node_activations: std::collections::HashMap<u32, f32> = std::collections::HashMap::new();
        let mut trace = Vec::new();

        let node_map: std::collections::HashMap<u32, &Node> = self.kb.nodes.iter().map(|n| (n.id, n)).collect();

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
            let mut next_activations = std::collections::HashMap::new();
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
                let node = self.kb.nodes.iter().find(|n| n.id == *node_id && n.kind == NodeKind::Solution)?;
                let solution = self.kb.solutions.iter().find(|s| s.node_id == *node_id)?;
                Some((score, node.clone(), solution.clone()))
            })
            .collect();

        solutions.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

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

    pub fn tokenise(&self, input: &str) -> Vec<String> {
        // Stop words to discard.
        const STOP: &[&str] = &[
            "a", "an", "the", "is", "it", "i", "my", "me", "we", "our", "you", "your",
            "do", "does", "did", "am", "are", "was", "were", "be", "been", "being",
            "get", "got", "have", "has", "had", "not", "no", "so", "to", "for", "of",
            "in", "on", "at", "by", "or", "and", "but", "if", "that", "this", "with",
            "from", "when", "why", "how", "what", "where", "can", "will", "would", "should",
            "keep", "getting", "keep", "always", "still", "just", "even", "only", "also",
        ];

        input
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty() && !STOP.contains(t))
            .map(|t| t.to_string())
            .collect()
    }
}
