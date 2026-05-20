mod cli;
mod engine;
mod knowledge;
mod model;

use std::path::Path;
use engine::Engine;

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
    let engine = Engine::new(kb);
    let result = engine.query(query, explain);

    if explain {
        println!("\nActivation trace:");
        // Group trace by hop
        let mut max_hop = 0;
        for step in &result.trace {
            if step.hop > max_hop {
                max_hop = step.hop;
            }
        }

        for (label, score, kind) in &result.seeds {
            println!("  {:15} {:.2}  [{}]", label, score, kind);
        }

        for hop in 1..=max_hop {
            for step in &result.trace {
                if step.hop == hop {
                    println!(
                        "  → {:20} {:.2} × {:.2} × 0.85 = {:.3}  [{}]",
                        step.dst_label,
                        step.src_activation,
                        step.edge_weight,
                        step.dst_activation,
                        step.dst_kind
                    );
                }
            }
        }
        println!();
    }

    match result.confidence {
        engine::ConfidenceLevel::High | engine::ConfidenceLevel::Medium => {
            if let Some((_, _, solution)) = result.top_solution {
                println!("{}", solution.text);
            }
        }
        engine::ConfidenceLevel::Low => {
            if let Some((score, _, _)) = result.top_solution {
                println!("Top solution (score {:.2} < θ_a 0.75): threshold not met — entering clarification", score);
            } else {
                println!("threshold not met — entering clarification");
            }
        }
        engine::ConfidenceLevel::Unknown => {
            println!("no candidates reached activation");
        }
    }
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

