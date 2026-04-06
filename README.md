# Engram

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

> *Engrams are defined as the physical changes in brain state induced by an
> event, serving as the memory trace.*

Edge weight updates are the physical changes. Resolved sessions are the
events. The graph is the memory trace. Knowledge accumulates structurally,
not as stored text.

---

## What Engram is

Engram is **symbolic AI** — knowledge is explicit, structured, and
inspectable, not distributed across opaque weights. This is a deliberate
choice, not a limitation. For bounded domains it is the right architecture:
every decision is auditable, every weight is named, and the system improves
without retraining.

A deterministic reasoning kernel. Instead of predicting answers statistically,
`engram` navigates a directed graph of concepts, asks targeted **breaking
questions** (targeted clarifying questions that partition the solution space)
to resolve ambiguity, reinforces correct reasoning paths through session
feedback, and emits typed **action contracts** that a separate execution layer
validates and runs.

At its core: a defined set of contexts and actions. Learning is finding the most
reliable path from an initial context to the right action. The system discovers
connections between things already in the graph — it cannot invent new knowledge.
Factual expansion requires a human to add new nodes and edges. This is intentional:
the original goal was a dialogue system with a hard domain boundary that stays
reliably correct within it, rather than a general system that occasionally hallucinates
outside it.

The design makes specific trade-offs that most AI tooling deliberately avoids:

| Requirement | Small LLM / fine-tuned model | Engram |
| --- | --- | --- |
| Same input → guaranteed same output | No — stochastic by design | Yes — deterministic graph traversal |
| Full reasoning trace, auditable to each step | No | Yes — every node and edge is named |
| Runs fully offline, no runtime dependency | Needs runtime / server | Yes — single binary, no network |
| Improves from session feedback without retraining | No — requires new fine-tune | Yes — edge weights update in real time |
| Stores patterns, never raw content | Depends on deployment | Structural — raw data never exists in transmittable form |
| Domain knowledge independently ownable per team | No — entangled in weights | Yes — separate graph files, swappable |

---

## Use Cases

**LLM agent mesh** — the highest-value deployment is Engram as the
deterministic first pass in front of an LLM:

```text
Query arrives
  │
  ▼
Engram graph activation
  │
  ├── High confidence ──► Answer directly   (no API call, microseconds)
  │
  └── Low confidence / novel ──► Escalate to LLM
          │
          └── Structured handoff: graph path + ruled-out candidates
              + confidence state (not raw conversation)
                    │
                    ▼
              LLM response → reinforces the graph for next time
```

In a well-trained bounded domain, Engram handles the majority of queries
without any model call. The LLM only sees genuinely novel cases — and each
one it resolves teaches the graph, making the next similar query cheaper.
A fleet of specialist graphs coordinated by a router is a deterministic
sparse Mixture of Experts: large total knowledge, small per-query compute,
every routing decision auditable.

**Privacy-preserving collective memory** — every resolved session compresses
into the graph as edge weight updates, never as stored conversation or
attributed records. Teams accumulate knowledge structurally: incident
runbooks distilled from resolutions, onboarding paths that outlast any wiki,
CI/CD triage patterns learned from real failures. Attribution is structurally
absent — not scrubbed, never recorded.

The same mechanism produces evidence-based technical debt maps. Component
nodes accumulate weight proportional to how often they appear in real incident
paths — not cyclomatic complexity, but confirmed fault frequency. A module
that appears in 87% of auth-related resolutions is a stronger refactoring
target than anything static analysis can identify. Latent node discovery
surfaces hidden coupling between components that co-activate across incidents
without a documented connection — structural problems the team never
explicitly named.

**Industrial domain agent** — bounded, high-stakes domains where determinism
and auditability are regulatory requirements: medical triage routing,
financial compliance screening, infrastructure fault isolation. The graph
stores the domain; the policy engine enforces who can trigger which actions;
the execution layer is completely separate. Also the right choice wherever
an LLM of any size is not permitted: air-gapped, safety-critical, or
regulated environments.

**Hierarchical distributed aggregation** — local Engram instances (including
browser-embedded WASM) emit compact graph deltas — not raw events, not
identifying data — to server instances that merge them upward. What travels
between nodes is edge weight adjustments, never raw user actions. Structural
differential privacy: the signal is the graph change, not the event. Product
analytics without telemetry pipelines; local-first apps contributing to a
global knowledge pool without central data collection.

**MCP server — knowledge database for LLM agents** — today's LLMs carry
memory in context windows (resets every conversation) or flat files (static,
unstructured). Engram exposed as an MCP tool gives any LLM agent access to
persistent, confidence-weighted, self-improving knowledge accumulated across
thousands of sessions. The LLM calls `engram.query()` mid-reasoning and
receives a typed reasoning path — confidence score, ruled-out candidates,
resolved dimensions — not a text chunk. Multiple agents sharing one Engram
instance share the *graph* (compressed patterns), never raw conversations.
The graph learns from every LLM-confirmed answer, so queries that initially
required model reasoning eventually resolve from Engram alone.

[docs/use_cases.md](docs/use_cases.md) documents all 17 deployment contexts.

---

## How the Graph Learns — Privacy by Architecture

User input text is discarded at the tokeniser boundary — it never enters any
storage layer. The tokeniser maps input tokens to node IDs and drops the text.
Everything downstream — propagation, reinforcement, weak memory, session records
— operates exclusively on node IDs. This is a structural guarantee, not a
sanitisation policy: there is no code path by which input text could reach storage.

After each session, Engram updates the graph based on outcome:

```text
Session confirmed →  edge weights on the successful path increase
                     w' = w + α(1 - w)   ← asymptotic, never saturates

Session rejected  →  edge weights on the failed path decrease
                     w' = w - α·w        ← proportional decay

Repeated enough times:
  high-confidence paths resolve without any questions asked
  low-confidence paths ask a breaking question to narrow the space
  failed paths accumulate in weak memory for later correction
```

What the graph stores after each session is a node activation pattern and an
outcome — not words, not who was involved, not what was typed. After 30 engineers
hit the same error and confirm the same fix:

```text
error=timeout + service=auth
  → CheckConnectionPool  [weight: 0.91, n=34]
```

The 34 people who contributed that weight are structurally absent — not
scrubbed, never recorded. The graph absorbed the outcome and discarded
the participants.

**Disagreement is handled gracefully.** Conflicting resolutions accumulate
as weak memory entries. A path earns high confidence only after repeated
independent confirmation — minority opinions stay provisional rather than
corrupting the shared graph.

**Latent structure emerges automatically.** When multiple sessions
independently co-activate the same two concepts without a direct connection,
the system creates a new hidden node between them — surfacing shared patterns
the team never explicitly documented.

This applies wherever a group works through the same problem space repeatedly:
incident response, onboarding, compliance routing, CI/CD triage.

---

## Design Principles

- **Deterministic** — same input always produces the same reasoning path
- **Explainable** — every answer shows exactly which nodes and edges led to it
- **Incremental learning** — session feedback updates edge weights in real time, no retraining
- **Offline** — no network, no API key, no model server
- **Composable** — domain knowledge in separate graph files, loadable and swappable independently
- **Action-first** — solution nodes carry typed contracts; execution layer strictly separated
- **Goal-aware** — multi-step goals span multiple exchanges with mid-conversation revision support
- **Escalation-ready** — structured context exported for handoff when confidence falls below threshold

---

## Prior Art

Several existing systems overlap with parts of Engram:

| System | What it shares | What it lacks |
| --- | --- | --- |
| **Drools / RETE rule engines** | Deterministic, auditable, fires typed actions | No dialogue layer, no graph traversal, salience is hand-tuned not learned |
| **Rasa** | Task-oriented dialogue, story graphs, slot-filling (breaking question analog) | Stores utterances, requires full retraining, not offline-first |
| **AIML / Pandorabots** | Deterministic pattern-match dialogue | No weight learning, no graph navigation |
| **Bayesian belief networks** | Weighted directed graph, deterministic inference | No dialogue layer, no action contracts |
| **OpenCyc / ResearchCyc** | Closed-world knowledge graph, hard factual boundary, offline | No learning, no dialogue |

The combination that does not exist elsewhere: a dialogue layer with structural privacy (no text stored at any layer), incremental weight learning without retraining, breaking questions as a first-class graph traversal primitive, and a hard knowledge boundary as an architectural guarantee rather than policy. Engram sits at the intersection of expert system, task-oriented dialogue, and reinforcement-learned policy graph — a combination shaped by constraints that existing tools treat as optional.

---

## Documentation

| Document | What it covers |
| --- | --- |
| [use_cases.md](docs/use_cases.md) | 17 deployment contexts with strategic priority — start here for the "why" |
| [proposal.md](docs/proposal.md) | Design overview, motivation, core concept, and index to all spec files |
| [architecture.md](docs/architecture.md) | Data structures, query pipeline, activation-as-attention |
| [disambiguation.md](docs/disambiguation.md) | Breaking questions, path labeling, goal tracking |
| [learning.md](docs/learning.md) | Reinforcement learning, latent discovery, weak memory, user profiles |
| [roadmap.md](docs/roadmap.md) | All 13 development phases with checkpoints and deliverables |
| [future.md](docs/future.md) | System comparison vs LLMs, future directions, sparse MoE framing |
| [storage.md](docs/storage.md) | CLI behavior and knowledge file layout |
| [metrics.md](docs/metrics.md) | Sizing budget and production outcome metrics |

---

## Status

**Phase 0 complete.** The Rust binary compiles and runs. Data structures,
file I/O, and the knowledge directory layout are in place.

**Next:** Phase 1 (keyword lookup against seed data) and Phase 2 (graph
activation and propagation) — the system becomes demonstrably useful at
Phase 2, navigating a real domain graph with a full reasoning trace.

See [docs/roadmap.md](docs/roadmap.md) for all 13 phases and checkpoints.

---

## Quick Start

**Requirements:** Rust 1.85+ (`rustup` recommended)

```sh
git clone https://github.com/dominikj111/Engram.git
cd Engram/app
cargo build
cargo run
```

```text
engram v0.1.0 — knowledge loaded: 0 nodes, 0 edges
engram> Hello
[phase 0] reasoning not yet implemented
query received: Hello
engram> exit
Goodbye.
```

The reasoning is a stub at Phase 0 — that is expected. The CLI surface,
data structures, file I/O, and knowledge directory layout are already in
place. The subcommands reflect the full architecture:

```text
$ cargo run -- --help

Usage: engram [OPTIONS] [QUERY] [COMMAND]

Commands:
  history      Show the last N sessions from history
  weak         List all unresolved weak memory entries
  latent       List all discovered latent nodes
  provisional  List all provisional (unconfirmed) nodes
  audit        Show bias audit: dominant edges and staleness report

Options:
  --explain           Print the full reasoning trace for each answer
  --knowledge-dir     Path to the knowledge directory (default: ./knowledge)
```

`weak`, `latent`, `provisional`, and `audit` are stubs today — each one
maps to a phase in [docs/roadmap.md](docs/roadmap.md). The architecture is
built to be filled in, not redesigned.

---

## Influences

- **Spreading activation** — Collins & Loftus (1975) — the activation propagation formula is a direct implementation
- **Rescorla-Wagner learning rule** — Rescorla & Wagner (1972) — the edge weight update formula `w' = w + α(1-w)` is a direct implementation
- **ACT-R cognitive architecture** — Anderson (1983) — activation thresholds, declarative memory chunks, learned associations across sessions
- **Case-Based Reasoning** — Aamodt & Plaza (1994) — the path cache is CBR
- **ConceptNet** — graph-based knowledge representation
- **Task-oriented dialogue systems** — breaking questions, goal tracking, action contracts, escalation payloads
- **Sparse Mixture of Experts** — the specialist swarm architecture is formally equivalent to sparse MoE

Not a clone of any of them — a deliberate combination shaped by the hard
constraints: deterministic output, full auditability, offline operation, no
retraining cycle.
