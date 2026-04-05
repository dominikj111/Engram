use clap::{Parser, Subcommand};

/// Engram — deterministic reasoning kernel. Sparse attention over a knowledge graph, without the GPU.
#[derive(Debug, Parser)]
#[command(
    name = "engram",
    version,
    about = "Deterministic reasoning kernel — sparse attention over a knowledge graph, without the GPU"
)]
pub struct Args {
    /// Print the full reasoning trace for each answer.
    #[arg(long, global = true)]
    pub explain: bool,

    /// Path to the knowledge directory (default: ./knowledge).
    #[arg(long, default_value = "knowledge")]
    pub knowledge_dir: String,

    /// Query to answer in single-shot mode.  Omit to enter interactive mode.
    pub query: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Optional sub-commands available alongside single-query and interactive modes.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show the last N sessions from history (default: 10).
    History {
        #[arg(default_value_t = 10)]
        n: usize,
    },
    /// List all unresolved weak memory entries.
    Weak,
    /// List all discovered latent nodes.
    Latent,
    /// List all provisional (unconfirmed) nodes.
    Provisional,
    /// Show bias audit: dominant edges and staleness report.
    Audit,
}
