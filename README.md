# chattie

A deterministic conversational reasoning kernel — designed to be the
explicit, auditable, fallback-safe layer inside larger orchestration systems.

Instead of predicting answers statistically, `chattie` navigates a directed
graph of concepts, asks targeted **breaking questions** to resolve ambiguity,
reinforces correct reasoning paths through session feedback, and emits typed
**action contracts** that a separate execution layer validates and runs.
It operates entirely offline, fits under 50 MB, and produces a full reasoning
trace for every decision.

---

## System Architecture

```text
┌─────────────────────────────────────────────────┐
│                 Interface Layer                  │
│   CLI · Web chat · Voice · HTTP API · Slack      │
│        (renders ResponseEnvelope per surface)    │
└──────────────────────┬──────────────────────────┘
                       │  query / event
                       ▼
┌─────────────────────────────────────────────────┐
│           Interaction Orchestrator               │
│  Goal tracking · UI context memory              │
│  Parameter resolution pipeline                  │
│  Urgency / priority scoring                     │
│  Noise handling / fuzzy activation              │
└──────────────────────┬──────────────────────────┘
                       │  normalised tokens + session state
                       ▼
┌─────────────────────────────────────────────────┐
│         Graph Reasoning Engine          ◄── core │
│  Activation propagation · Breaking questions    │
│  Confidence state machine · Path labeling       │
│  Reinforcement learning · Latent discovery      │
│  Weak memory · User profile                     │
└──────────────────────┬──────────────────────────┘
                       │  action contract + params
                       ▼
┌─────────────────────────────────────────────────┐
│            Policy Engine                         │
│  Permission check · Rate limits                 │
│  Confirmation requirements · Rollback flags     │
└──────────────────────┬──────────────────────────┘
                       │  validated contract
                       ▼
┌─────────────────────────────────────────────────┐
│          Action Execution Layer                  │
│  CheckLineStatus · RebootRouter · ...           │
│  (typed contracts; implementations live here)   │
└──────────────────────┬──────────────────────────┘
                       │  result / event
                       ▼
┌─────────────────────────────────────────────────┐
│              Backend Systems                     │
│   CRM · Network monitoring · Billing · Devices  │
└─────────────────────────────────────────────────┘
```

The graph reasoning engine is the core. Every other layer is thin, swappable,
and can be added incrementally — the engine is useful from Phase 2 onward
even without the orchestrator or execution layers present.

---

## What this is not

- **Not a chatbot wrapper around an LLM.** There is no language model. Every
  reasoning step is an explicit graph traversal you can inspect and audit.
- **Not a fixed decision tree.** The graph learns from every interaction —
  edge weights update in real time, new concepts emerge automatically.
- **Not advice-only.** Solution nodes can carry typed action contracts.
  The system selects the action; a separate executor runs it safely.

## What the engine alone cannot yet do

The graph reasoning engine is the cognitive core. The surrounding layers in
the architecture diagram above are required to compete with production
chatbots. The current implementation focus is the engine. The gaps, in
priority order:

| Gap                     | What it requires                                  | Spec    |
| ----------------------- | ------------------------------------------------- | ------- |
| Structured output       | `ResponseEnvelope` wrapping every response        | §3.5    |
| Smart parameter filling | `ResolutionChain` before asking the user          | §3.4    |
| Permission / safety     | `PolicyEngine` before execution                   | §3.6    |
| Explicit uncertainty    | `ConfidenceLevel` state machine                   | §4.5    |
| Multi-step goals        | `Goal` struct with revision support               | §7.5    |
| UI consistency          | `UIContextRecord` per session turn                | §11.3   |
| Messy input tolerance   | Fuzzy layers + partial activation path            | §15.5   |
| Production metrics      | Resolution / escalation / friction rates          | §17.5   |
| System-initiated turns  | Event-driven entry point alongside queries        | §20.11  |
| Escalation quality      | `EscalationPayload` with full structured handoff  | §3.4    |

Each row is independently addable — none depends on the others being present.

---

## Status

**Phase 0 — skeleton.** The Rust binary compiles and runs. Data structures,
file I/O, and the knowledge directory layout are the current focus.
See [`docs/proposal.md`](docs/proposal.md) for the full design and 13-phase roadmap.

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
chattie --weak                   # list unresolved uncertain answers
chattie --latent                 # list auto-discovered concept nodes
chattie --goals                  # show open and recent goals
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
- **Action-first** — solution nodes carry typed contracts; the execution layer
  is strictly separated from the reasoning layer
- **Goal-aware** — multi-step goals span multiple exchanges, carry context
  forward, and support mid-conversation revision
- **Escalation-ready** — when confidence falls below threshold, structured
  session context is exported for human handoff rather than returning nothing

---

## Vertical Slice: Internet Connectivity Diagnosis

The recommended first deployment is a single narrow domain that exercises
every architectural layer with real constraints:

```text
User: "my internet keeps dropping"

[activation] connectivity_issue: 0.90

? Does the problem affect all devices, or just one?  [scope_dimension]
> all devices

[branch: network_wide_fault]

? Has the connection been dropping intermittently or completely absent?  [duration_dimension]
> intermittent

[path: intermittent_fault → line_fault]
[action contract selected: CheckLineStatus]
  → account_id: <from session context>

[execution layer] CheckLineStatus(account_id: "ACC-001")
  → sync_status: "retrain loop", signal_dbm: -78, error_count: 312

Answer: Line quality is degraded (signal: -78 dBm, 312 errors).
        An engineer visit has been scheduled — reference: ENG-20260401.
[goal: diagnose_connectivity → Resolved]
```

This slice covers activation propagation, breaking questions, action
contracts, execution layer separation, goal tracking, and escalation — all
within 3–5 graph nodes and 4 action definitions. The same engine then serves
billing, device provisioning, and account management by loading different
persona graph files.

---

## Prior Art and Influences

Spreading activation (Collins & Loftus 1975), Bayesian Knowledge Tracing
(Corbett & Anderson 1994), ConceptNet, and task-oriented dialogue systems.
Not a clone of any of them — a deliberate combination shaped around the
constraints of a fast offline CLI tool.
