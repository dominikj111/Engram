# Engram

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

> *Engrams are defined as the physical changes in brain state induced by an
> event, serving as the memory trace.*

**Jump to:** [What it is](#what-engram-is) · [Use cases](#use-cases) · [How it learns](#how-the-graph-learns--privacy-by-architecture) · [Quick start](#quick-start) · [Prior art](#prior-art)

---

## What Engram is

Engram is a **deterministic reasoning kernel** — symbolic AI with hard boundaries and
fluid internals. The nodes and actions are fixed by design; what learns over time are
the connections between them. Given a context, Engram navigates a directed graph of
concepts, asks targeted **breaking questions** to resolve ambiguity, and emits typed
**action contracts** that a separate execution layer runs. Every path is auditable,
every weight is named, and the system improves without retraining — not by inventing
new knowledge, but by finding more reliable routes through what it already knows.

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

[docs/use_cases.md](docs/use_cases.md) documents all 10 deployment contexts.

---

## How the Graph Learns — Privacy by Architecture

User input text is discarded at the tokeniser boundary — it never enters any
storage layer. This is a structural guarantee, not a sanitisation policy.

After each session, edge weights on the confirmed path increase; weights on
rejected paths decay. What the graph stores is a node activation pattern and
an outcome — not words, not who was involved, not what was typed. After 30
engineers hit the same error and confirm the same fix:

```text
error=timeout + service=auth
  → CheckConnectionPool  [weight: 0.91, n=34]
```

The 34 people who contributed that weight are structurally absent — not
scrubbed, never recorded. Conflicting resolutions stay provisional until
independently confirmed. Hidden relationships between concepts surface
automatically as the session population grows.

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
- **Configurable learning** — each graph deployment chooses whether to learn from live traffic or stay frozen

---

## Graph Deployment Modes

The graph is a file on disk. Write-back is a deliberate per-deployment choice, not a
system default. This gives two distinct operating modes:

**Continuous learning** — sessions write back to the graph in real time. Edge weights
shift with every confirmed or rejected outcome. The right choice for collective memory,
LLM agent mesh, and MCP deployments where improving from live traffic is the point.

**Frozen / versioned** — the engine reasons against a static graph and never mutates
it. Learning happens offline: session outcomes accumulate in a staging environment, a
new graph version is validated and promoted deliberately. The right choice for
industrial agents, compliance routing, and business logic runners where stability and
auditability matter more than continuous improvement — the graph behaves like versioned
application code.

The same engine supports both. Which mode a deployment uses is a configuration
decision, not an architectural one.

More precisely, there are three independently lockable axes — context nodes,
actions, and graph learning — giving eight deployment configurations from fully
frozen to fully adaptive. See [architecture.md §3.7](docs/architecture.md) for
the full matrix and the provisional node mechanism that makes open axes safe.

**Inspecting the graph:** the graph files are plain JSON — readable with any
tool today. The planned Phase 14 milestone adds a visual connectome inspector:
load a knowledge directory, watch activation propagate from a query to a
solution node, click any edge to inspect its weight and session history.
See [roadmap.md](docs/roadmap.md) for details.

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
| [use_cases.md](docs/use_cases.md) | 10 deployment contexts with strategic priority — start here for the "why" |
| [proposal.md](docs/proposal.md) | Design overview, motivation, core concept, and index to all spec files |
| [architecture.md](docs/architecture.md) | Data structures, query pipeline, activation-as-attention |
| [disambiguation.md](docs/disambiguation.md) | Breaking questions, path labeling, goal tracking |
| [learning.md](docs/learning.md) | Reinforcement learning, latent discovery, weak memory, user profiles |
| [roadmap.md](docs/roadmap.md) | All 15 development phases (Phase 0–14) with checkpoints and deliverables |
| [future.md](docs/future.md) | System comparison vs LLMs, future directions, sparse MoE framing |
| [storage.md](docs/storage.md) | CLI behavior and knowledge file layout |
| [metrics.md](docs/metrics.md) | Sizing budget and production outcome metrics |

---

## Status

**Phase 1 complete.** The binary compiles, loads a real knowledge graph, and
answers HTTP/API queries by keyword lookup with an optional reasoning trace.

**Next:** Phase 2 (graph activation and propagation) — the system navigates
the graph by spreading activation through edges, producing ranked candidates
with confidence scores rather than direct keyword matches.

The Rust binary is the reference implementation. The knowledge file format
(JSON) and the reasoning spec are language-agnostic — implementations in any
language are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

See [docs/roadmap.md](docs/roadmap.md) for all 15 phases (Phase 0–14) and checkpoints.

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
engram v0.1.0 — knowledge loaded: 19 nodes, 17 edges
engram> 401 unauthorized
401 Unauthorized — the request lacks valid authentication. Check that your
token or API key is present, not expired, and sent in the correct header
(usually Authorization: Bearer <token>).

engram> cors blocked
CORS error — the browser blocked the request because the server did not
include the required Access-Control-Allow-Origin header. Add CORS middleware
on the server, or configure it to allow your origin explicitly.

engram> my api keeps getting 429
429 Too Many Requests — the rate limit has been exceeded. Back off and retry
after the duration in the Retry-After header.

engram> exit
Goodbye.
```

A seed HTTP/API knowledge graph ships with the repo — 19 nodes, 17 edges,
covering the most common status codes, CORS, timeouts, rate limits, and SSL
errors. Pass `--explain` at startup to see the reasoning trace on every answer:

```text
$ cargo run -- --explain
engram> cors blocked
CORS error — ...

  path:  fix_cors
  score: 3.76
  via:   fix_cors → cors
```

The subcommands reflect the full architecture:

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
