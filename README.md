# chattie

A lightweight, deterministic, self-improving CLI chatbot powered by a
weighted context graph — no LLM, no GPU, fully explainable, learns from
every interaction.

Instead of predicting answers statistically, `chattie` navigates a directed
graph of concepts, asks targeted **breaking questions** to resolve ambiguity,
and reinforces correct reasoning paths through session feedback. It runs
entirely offline, fits under 50 MB, and produces a full reasoning trace for
every answer.

---

## Status

**Phase 0 — skeleton.** The Rust binary compiles and runs. Data structures,
file I/O, and the knowledge directory layout are the current focus.
See [`proposal.md`](proposal.md) for the full design and 13-phase roadmap.

---

## Quick Start

**Requirements:** Rust 1.85+ (`rustup` recommended)

```sh
# clone
git clone https://github.com/your-username/chattie.git
cd chattie/app

# build
cargo build

# run
cargo run
```

---

## Project Layout

```text
chattie/
  app/              — main application source
    src/main.rs
    Cargo.toml
  proposal.md       — full design document and development roadmap
  README.md
```

As the project progresses, a `knowledge/` directory will appear alongside
`app/` containing the graph data files (`nodes.json`, `edges.json`,
`paths.json`, etc.).

---

## Planned CLI

```sh
chattie                          # interactive loop
chattie "why rust borrow error"  # single query
chattie --explain "..."          # show full reasoning path
chattie --history 10             # last 10 sessions
chattie --audit                  # graph health report
```

---

## Design Principles

- **Deterministic** — same input always produces the same reasoning path
- **Explainable** — every answer can show exactly which nodes and edges led to it
- **Incremental learning** — session feedback updates edge weights in real time,
  no retraining step
- **Offline** — no network, no API key, no model server
- **Composable** — domain knowledge is stored as separate graph files that
  can be loaded, combined, and swapped independently

---

## Prior Art and Influences

Spreading activation (Collins & Loftus 1975), Bayesian Knowledge Tracing
(Corbett & Anderson 1994), ConceptNet, and task-oriented dialogue systems.
Not a clone of any of them — a deliberate combination shaped around the
constraints of a fast offline CLI tool.
