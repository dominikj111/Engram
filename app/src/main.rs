mod cli;
mod knowledge;
mod model;

use std::path::Path;

use clap::Parser;
use rustyline::{DefaultEditor, error::ReadlineError};

use cli::{Args, Command};
use knowledge::KnowledgeBase;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args = Args::parse();
    let knowledge_dir = Path::new(&args.knowledge_dir);

    let kb = match KnowledgeBase::load(knowledge_dir) {
        Ok(kb) => kb,
        Err(e) => {
            eprintln!("error: failed to load knowledge base from '{}': {e}", knowledge_dir.display());
            std::process::exit(1);
        }
    };

    match &args.command {
        Some(Command::History { n }) => cmd_history(&kb, *n),
        Some(Command::Weak) => cmd_weak(&kb),
        Some(Command::Latent) => cmd_latent(&kb),
        Some(Command::Provisional) => cmd_provisional(&kb),
        Some(Command::Audit) => cmd_audit(&kb),
        None => {
            if let Some(query) = &args.query {
                run_single_query(&kb, query, args.explain);
            } else {
                run_interactive(&kb, args.explain);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Interactive REPL
// ---------------------------------------------------------------------------

fn run_interactive(kb: &KnowledgeBase, explain: bool) {
    println!(
        "engram v{VERSION} — knowledge loaded: {} nodes, {} edges",
        kb.node_count(),
        kb.edge_count()
    );

    let mut rl = DefaultEditor::new().expect("failed to initialise line editor");

    loop {
        match rl.readline("engram> ") {
            Ok(line) => {
                let input = line.trim().to_string();
                if input.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(&input);

                match input.as_str() {
                    "exit" | "quit" | ":q" => {
                        println!("Goodbye.");
                        break;
                    }
                    "help" | ":help" => print_help(),
                    query => run_single_query(kb, query, explain),
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C
                println!("^C");
                break;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D
                break;
            }
            Err(err) => {
                eprintln!("error: {err}");
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Single-query mode
// ---------------------------------------------------------------------------

fn run_single_query(kb: &KnowledgeBase, query: &str, explain: bool) {
    let tokens = tokenise(query);

    // Find nodes whose label matches any token.
    let mut matches: Vec<(f32, &model::Node)> = kb
        .nodes
        .iter()
        .filter_map(|node| {
            let label_tokens: Vec<&str> = node.label.split('_').collect();
            let score = tokens
                .iter()
                .filter(|t| {
                    node.label.contains(t.as_str())
                        || label_tokens.iter().any(|lt| lt.starts_with(t.as_str()))
                        || node.tags.iter().any(|tag| tag == t.as_str())
                })
                .count() as f32;
            if score > 0.0 { Some((score, node)) } else { None }
        })
        .collect();

    matches.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // Follow edges from matched concept nodes to find solution nodes.
    let mut solution_scores: std::collections::HashMap<u32, f32> = std::collections::HashMap::new();
    for (score, node) in &matches {
        // If the matched node is itself a solution, score it directly.
        if node.kind == model::NodeKind::Solution {
            *solution_scores.entry(node.id).or_default() += score * 1.5;
        }
        // Also follow outgoing edges.
        for edge in kb.edges.iter().filter(|e| e.src == node.id) {
            *solution_scores.entry(edge.dst).or_default() += score * edge.weight * edge.confidence;
        }
    }

    // Collect solution nodes by score.
    let mut solutions: Vec<(f32, &model::Node, &model::Solution)> = solution_scores
        .iter()
        .filter_map(|(node_id, score)| {
            let node = kb.nodes.iter().find(|n| n.id == *node_id && n.kind == model::NodeKind::Solution)?;
            let solution = kb.solutions.iter().find(|s| s.node_id == *node_id)?;
            Some((*score, node, solution))
        })
        .collect();

    solutions.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    if let Some((score, node, solution)) = solutions.first() {
        println!("{}", solution.text);
        if explain {
            println!();
            println!("  path:  {}", node.label);
            println!("  score: {:.2}", score);
            let activated: Vec<&str> = matches.iter().take(3).map(|(_, n)| n.label.as_str()).collect();
            println!("  via:   {}", activated.join(" → "));
        }
    } else if tokens.is_empty() {
        println!("No input — type a question or 'help'.");
    } else {
        println!("No match found for: {}", tokens.join(", "));
        println!("Try keywords like: 401, 403, 404, 500, cors, timeout, rate limit, ssl");
    }
}

// ---------------------------------------------------------------------------
// Tokeniser (Phase 1 — simple keyword extraction)
// ---------------------------------------------------------------------------

fn tokenise(input: &str) -> Vec<String> {
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

// ---------------------------------------------------------------------------
// Sub-commands (stubs for future phases)
// ---------------------------------------------------------------------------

fn cmd_history(kb: &KnowledgeBase, n: usize) {
    let sessions = &kb.sessions;
    if sessions.is_empty() {
        println!("No sessions recorded yet.");
        return;
    }
    let start = sessions.len().saturating_sub(n);
    for s in &sessions[start..] {
        let questions: Vec<String> = s.breaking_questions_asked.iter().map(|id| id.to_string()).collect();
        println!(
            "{}  {}  {}  questions: [{}]",
            s.session_id,
            s.path_labels.join(", "),
            s.outcome,
            questions.join(", ")
        );
    }
}

fn cmd_weak(kb: &KnowledgeBase) {
    if kb.weak_memory.is_empty() {
        println!("No weak memory entries.");
        return;
    }
    for e in &kb.weak_memory {
        println!("{}  [{}]  nodes: {:?}  →  attempted: {}", e.id, e.status, e.activated_nodes, e.attempted_path);
    }
}

fn cmd_latent(kb: &KnowledgeBase) {
    use model::NodeKind;
    let latent: Vec<_> = kb.nodes.iter().filter(|n| n.kind == NodeKind::Latent).collect();
    if latent.is_empty() {
        println!("No latent nodes discovered yet.");
        return;
    }
    for n in latent {
        println!("{}  [latent]  tags: [{}]", n.label, n.tags.join(", "));
    }
}

fn cmd_provisional(kb: &KnowledgeBase) {
    let provisional: Vec<_> = kb.nodes.iter().filter(|n| n.tags.contains(&"unconfirmed".to_string())).collect();
    if provisional.is_empty() {
        println!("No provisional nodes pending.");
        return;
    }
    for n in provisional {
        println!("{}  [provisional]  tags: [{}]", n.label, n.tags.join(", "));
    }
}

fn cmd_audit(_kb: &KnowledgeBase) {
    println!("[phase 0] bias audit not yet implemented — available from phase 12");
}

// ---------------------------------------------------------------------------
// Help
// ---------------------------------------------------------------------------

fn print_help() {
    println!(
        r#"Commands:
  <query>       Ask a question
  help          Show this help
  exit / quit   Exit engram

CLI flags (pass before the query):
  --explain     Show reasoning trace
  --knowledge-dir <path>  Override knowledge directory

Sub-commands:
  engram history [N]   Last N sessions
  engram weak          Unresolved weak memory entries
  engram latent        Discovered latent nodes
  engram provisional   Provisional (unconfirmed) nodes
  engram audit         Bias audit report"#
    );
}

