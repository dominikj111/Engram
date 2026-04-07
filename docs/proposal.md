# Proposal: Engram — Deterministic Reasoning Kernel

**Status:** Draft v1.2  
**Goal:** A lightweight, deterministic, self-improving reasoning kernel — sparse attention over a knowledge graph, without the GPU. Breaking question decomposition, path labeling, and incremental reinforcement learning from confirmed sessions.  
**Constraints:** <100 MB memory, fully explainable, incremental learning, no external model dependency.

---

## 1. Objective

### 1.1 Motivation

Large language models are remarkable — and enormous. A typical deployment
consumes tens of gigabytes of weights, requires a GPU or a remote API call,
and re-reasons from scratch on every query. For many real-world use cases this
is unnecessary: human vocabulary for recurring problems is finite and
repetitive. The same questions get asked dozens of times — the same error, the
same workflow, the same diagnosis path. A system that has seen a question
resolved correctly once does not need billions of parameters to handle it the
second time.

Engram was designed around this observation. For bounded domains, a learned
graph under 100 MB handles the majority of queries offline in microseconds —
and improves automatically from every resolved session. The LLM remains
available for genuinely novel cases; it just stops being the default path for
everything.

### 1.2 Design Goals

Design a **lightweight deterministic reasoning kernel** that:

- runs as a **CLI application**
- navigates a **weighted context graph** to answer questions
- decomposes ambiguity using **breaking questions** with labeled branches
- tags and reuses **named context paths**
- **learns incrementally** from every interaction
- discovers **latent nodes** representing emergent concepts
- records **weak or uncertain answers** for later correction

The system evolves over time into a **self-organizing knowledge network**.

---

## 2. Core Concept

Instead of predicting answers statistically like an LLM, the system builds a
**semantic graph of context nodes**. A question becomes a **graph navigation problem**.

```text
User Question
      │
      ▼
Tokenize + Normalize
      │
      ▼
Context Node Activation
      │
      ▼
Activation Propagation
      │
      ▼
Path Selection
      │
      ├── High confidence ──► Answer
      │
      └── Ambiguous ──► Breaking Question
                              │
                              ▼
                        Labeled Branch
                              │
                              ▼
                           Answer
```

The key principle: every step of the reasoning is **explicit and auditable**.

---

## 2.5 Target Use Cases

A full description of deployment contexts is maintained in [use_cases.md](use_cases.md).
The five primary use cases are summarised here:

| Use Case | Key Benefit |
| --- | --- |
| **Team knowledge distillation** | Privacy-preserving collective memory — stores reasoning paths, never attribution |
| **Industrial domain agent** | Deterministic, auditable, offline-capable vertical deployment |
| **Voice assistant runtime** | Surface-agnostic `ResponseEnvelope`, numbered-choice breaking questions |
| **Multi-command orchestrator** | Policy-gated action dispatch with event-driven initiation |
| **Compressed chat memory** | Sub-linear growth, high signal-to-noise, no raw dialog stored |

The unifying property: the graph stores *what was confirmed as correct*, not *who said what
or when*. This makes the system suitable wherever factual substrate matters more than
conversational record.

---

## Documentation

| File | Contents |
| ---- | -------- |
| [architecture.md](architecture.md) | §1 Objective/Motivation, §2 Core Concept, §2.5 Use Cases summary, §3 Knowledge Representation (§3.1–§3.6), §4 Query Processing (§4.1–§4.5 including §4.3.1) |
| [disambiguation.md](disambiguation.md) | §5 Breaking Questions, §6 Context Path Labeling, §7 Clarification, §7.5 Goal Tracking (§7.5.1–§7.5.4) |
| [learning.md](learning.md) | §8 Learning from Interaction, §9 Reinforcement Strategy, §10 Latent Node Discovery, §11 Weak Answer Memory, §11.3 UI Context Memory, §11.5 User Profile |
| [storage.md](storage.md) | §12 CLI Behavior, §13 Memory Layout |
| [knowledge.md](knowledge.md) | §14 Initial Knowledge Base, §15 Automatic Context Expansion, §15.5 Noise Handling, §16 Context Bias |
| [metrics.md](metrics.md) | §17 Expected System Size, §17.5 Outcome Metrics |
| [roadmap.md](roadmap.md) | §18 Development Phases — all phases 0 through 14 including Phase 13 sub-sections and both summary tables |
| [future.md](future.md) | §19 System Comparison, §20 Future Directions (§20.1–§20.11 including §20.5.1), §21 Summary |
| [use_cases.md](use_cases.md) | Detailed deployment context descriptions for all five primary use cases |
