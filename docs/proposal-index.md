# Engram — Master Navigation Index

Quick-reference map for the full Engram design documentation.
Start at [proposal.md](proposal.md) for the hub overview and links to each file.

---

## architecture.md — Architecture and Data Structures

Covers the core data model, query processing pipeline, and the structural
relationship between Engram's activation propagation and transformer attention.

| Section | Purpose |
| ------- | ------- |
| §1.1 Motivation | Origin story — LLMs are too large for repetitive bounded domains |
| §1.2 Design Goals | What the system is designed to do |
| §2 Core Concept | Graph navigation diagram and one-liner |
| §2.5 Target Use Cases | Summary table of 5 deployment contexts |
| §3.1 Context Graph | Weighted directed graph — nodes are concepts, edges are transitions |
| §3.2 Node Structure | `Node` struct; `NodeKind` enum (Concept / Question / Solution / Latent / Escalation) |
| §3.3 Edge Structure | `Edge` struct with weight, confidence, usage count, path labels |
| §3.4 Solution Node Variants | `SolutionPayload` enum; `ResolutionChain` parameter pipeline; `EscalationPayload` handoff struct |
| §3.5 Response Envelope | `ResponseEnvelope` — structured output contract for all adapters (CLI / web / voice / API) |
| §3.6 Policy Engine | `PolicyEngine` — permission checks, rate limits, confirmation rules before any action executes |
| §4.1 Tokenization | Stop-word removal, stemming, alias map |
| §4.2 Context Activation | Matched tokens → initial activation scores on nodes |
| §4.3 Activation Propagation | Forward spread through edges: `a_target = a_source × w × λ` |
| §4.3.1 Propagation as Attention | Formal equivalence to sparse attention; specialist graphs as domain-specific attention patterns |
| §4.4 Candidate Ranking | Leaf `Solution` nodes ranked by accumulated activation |
| §4.5 Confidence State Machine | `ConfidenceLevel` enum (High/Medium/Low/Unknown) with explicit behaviour per state |

---

## disambiguation.md — Disambiguation and Goal Tracking

Covers how the system resolves ambiguity, labels reasoning paths, and tracks
multi-turn goals with urgency-based prioritisation.

| Section | Purpose |
| ------- | ------- |
| §5 Breaking Questions | Multi-branch decomposition; branch selection maximally separates candidates |
| §5.1 Purpose | When and why breaking questions are triggered |
| §5.2 Breaking Question Structure | `BreakingQuestion` and `Branch` structs |
| §5.3 Decomposition Strategy | How the system picks the best breaking question |
| §5.4 Question Labeling | Domain dimension labels on `Question` nodes |
| §6 Context Path Labeling | Named, tagged `ContextPath` records; three-tier tag taxonomy |
| §6.1 Path Definition | `ContextPath` struct |
| §6.2 Tag Taxonomy | Domain / Pattern / Scope tiers |
| §6.3 Path Cache | Active nodes matched against known paths → fast re-resolution |
| §6.4 Tag Propagation to Sessions | Session tag histories and their downstream uses |
| §6.5 Path Label Evolution | Stale path detection when edge confidence drops |
| §7 Clarification | Single yes/no confirmation when one candidate dominates but confidence is low |
| §7.5 Goal Tracking | Multi-step goals with `GoalStatus`, revision support, parallel sub-goals |
| §7.5.1 Goal Structure | `Goal` struct and `GoalStatus` enum |
| §7.5.2 Goal Revision | Mid-conversation reframing without restarting graph traversal |
| §7.5.3 Parallel Sub-Goals | Two unrelated high-confidence paths → two simultaneous sub-goals |
| §7.5.4 Urgency and Impact Scoring | `urgency:` and `impact:` tags drive priority when sub-goals compete |

---

## learning.md — Learning, Memory, and User Profiles

Covers how the system improves from every interaction, stores failures, and
adapts its behavior to individual users.

| Section | Purpose |
| ------- | ------- |
| §8 Learning from Interaction | How confirmed/rejected sessions update the graph |
| §9.1 Positive Reinforcement | Weight update formula on confirmed solutions |
| §9.2 Negative Reinforcement | Weight decay formula on rejected solutions |
| §9.3 Path-Level Reinforcement | Per-edge reinforcement scaled by path length |
| §10 Latent Node Discovery | Co-occurrence monitoring → auto-created hidden concept nodes |
| §10.1 Co-occurrence Monitoring | Pairwise session co-activation counters |
| §10.2 Similarity Score | Jaccard-normalized co-occurrence formula |
| §10.3 Latent Node Creation | Threshold, creation steps, and example |
| §11 Weak Answer Memory | Uncertain/rejected answers stored in `weak_memory.json` for later correction |
| §11.1 Storage Format | JSON record for weak memory entries |
| §11.2 Promotion to Main Graph | Five-step correction and reinforcement process |
| §11.3 UI Context Memory | Per-turn record of what was shown, clicked, dismissed — prevents repeated options |
| §11.5 User Profile | `UserProfile` with `SkillLevel`; drives question de-prioritisation and response verbosity |
| §11.5.1 Skill Level Derivation | Table mapping session signals to Novice / Intermediate / Expert |
| §11.5.2 Profile-Driven Routing | How profile shortcuts and verbosity adjustments work |

---

## storage.md — CLI Behavior and Memory Layout

Covers the three CLI modes and the full on-disk file layout.

| Section | Purpose |
| ------- | ------- |
| §12.1 Interactive Loop | Full interactive session example with activation trace |
| §12.2 Single Query Mode | One-shot command invocation |
| §12.3 Explanation Mode | `--explain` output format with path and tags |
| §13 Memory Layout | Directory structure: nodes / edges / paths / questions / solutions / weak_memory / sessions |
| §13.1 nodes.json | JSON schema example |
| §13.2 edges.json | JSON schema example |
| §13.3 paths.json | JSON schema example |

---

## knowledge.md — Knowledge Base, Context Expansion, and Context Bias

Covers the seed knowledge, how the system learns new vocabulary, how it handles
noisy real-world input, and how learned bias affects routing.

| Section | Purpose |
| ------- | ------- |
| §14 Initial Knowledge Base | Seed nodes, edges, breaking questions, and paths to start from |
| §15 Automatic Context Expansion | Unknown tokens → provisional nodes → promotion after 3 confirmations |
| §15.5 Real-World Noise Handling | Fuzzy matching (edit distance, n-gram, emotional strip); partial activation; incomplete info tolerance |
| §15.5.1 Partial Activation | Best-guess + correction loop for sparse token matches |
| §15.5.2 Fuzzy Token Matching | Three-layer fuzzy pipeline before provisional node creation |
| §15.5.3 Incomplete Information Tolerance | Deferred parameter resolution for action contracts |
| §16 Context Bias | High-weight edge dominance; exploration noise ε prevents lock-in |

---

## metrics.md — System Size and Outcome Metrics

Covers the memory budget and the production-facing metrics that indicate whether
the graph is actually solving problems.

| Section | Purpose |
| ------- | ------- |
| §17 Expected System Size | Component-by-component size budget; total < 45 MB |
| §17.5 Outcome Metrics | 8 production metrics (resolution rate, escalation rate, friction score, …); `--metrics` command |

---

## roadmap.md — Development Roadmap

Covers all development phases from skeleton to near-LLM quality, each with
deliverables, checkpoints, and inspectable artifacts.

| Phase | Capability |
| ----- | ---------- |
| 0 | Compilable skeleton, file I/O |
| 1 | Static keyword lookup from seed data |
| 2 | Graph activation and propagation |
| 3 | Single yes/no clarification |
| 4 | Multi-branch breaking questions |
| 5 | Named path recording with tags |
| 6 | Path-level cache (fast re-resolution) |
| 7 | Session recording with audit trail |
| 8 | Reinforcement learning |
| 9 | Weak answer memory |
| 10 | Latent node discovery |
| 11 | Automatic context expansion |
| 12 | Bias tuning and exploration noise |
| 13 | BM25 + n-grams + session carry-forward + composite answers |

Phase 13 sub-sections: §13.1 BM25 Retrieval, §13.2 N-gram Token Matching,
§13.3 Session Context Carry-Forward, §13.4 Composite Answer Assembly.
Both summary tables (Phases 0–12 and Phases 0–13) are included.

---

## future.md — System Comparison, Future Directions, and Summary

Covers the feature comparison with LLMs, eleven independent architectural
directions for after Phase 12, and the closing summary.

| Section | Purpose |
| ------- | ------- |
| §19 System Comparison | Feature table: this system vs. LLM (14 dimensions) |
| §20.1 Neural Embedding | Fuzzy token matching via word2vec/GloVe (~10–30 MB) |
| §20.2 Neural Re-ranker | Shallow MLP reorders graph-produced candidates; never overrides the graph |
| §20.3 Intent Classifier | Routes query to correct domain persona before activation |
| §20.4 Graph Distillation | Distil routing logic into tiny neural fast-path; graph remains authoritative |
| §20.5 Persona Graphs | Separable domain knowledge files; inspectable, swappable, composable. **Elevated to core architectural pattern.** |
| §20.5.1 Swarm as Sparse MoE | Formal equivalence to sparse Mixture of Experts; recursive router composition; LLM crossover point |
| §20.6 NL Answer Formatter | Optional tiny model for natural phrasing; `--raw` disables it |
| §20.7 Distributed Sharing | Export / merge persona graphs; weighted-average merge strategy |
| §20.8 Network Service | Extract engine as library; add HTTP + Discord/Slack/Teams/Web adapters |
| §20.9 Pre-neural ROI | BM25, n-grams, carry-forward, composite answers — ~300 LOC, zero deps |
| §20.10 Telecom Vertical Slice | Full connectivity diagnosis agent spec: 4 actions, breaking questions, layer validation table |
| §20.11 Event-Driven Core | `EngineInput::SystemEvent`; system initiates turns on outage/device/action events |
| §21 Summary | One-paragraph system description |

---

## use_cases.md — Deployment Use Cases

Detailed descriptions of the five primary deployment contexts. Referenced from
§2.5 in proposal.md and architecture.md, and from §20.5 in future.md.
