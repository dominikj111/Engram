# Proposal Index

Quick-reference map for [proposal.md](proposal.md).
Each entry shows the section, its one-line purpose, and the line number to
jump to directly.

---

## Core Data Structures (§3)

| Section | Purpose | Line |
| ------- | ------- | ---- |
| §3.1 Context Graph | Weighted directed graph — nodes are concepts, edges are transitions | 62 |
| §3.2 Node Structure | `Node` struct; `NodeKind` enum (Concept / Question / Solution / Latent / Escalation) | 74 |
| §3.3 Edge Structure | `Edge` struct with weight, confidence, usage count, path labels | 96 |
| §3.4 Solution Node Variants | `SolutionPayload` enum; `ResolutionChain` parameter pipeline; `EscalationPayload` handoff struct | 119 |
| §3.5 Response Envelope | `ResponseEnvelope` — structured output contract for all adapters (CLI / web / voice / API) | 215 |
| §3.6 Policy Engine | `PolicyEngine` — permission checks, rate limits, confirmation rules before any action executes | 269 |

---

## Query Processing Pipeline (§4)

| Section | Purpose | Line |
| ------- | ------- | ---- |
| §4.1 Tokenization | Stop-word removal, stemming, alias map | 321 |
| §4.2 Context Activation | Matched tokens → initial activation scores on nodes | 331 |
| §4.3 Activation Propagation | Forward spread through edges: `a_target = a_source × w × λ` | 341 |
| §4.4 Candidate Ranking | Leaf `Solution` nodes ranked by accumulated activation | 357 |
| §4.5 Confidence State Machine | `ConfidenceLevel` enum (High/Medium/Low/Unknown) with explicit behaviour per state | 370 |

---

## Disambiguation (§5–§7)

| Section | Purpose | Line |
| ------- | ------- | ---- |
| §5 Breaking Questions | Multi-branch decomposition; branch selection maximally separates candidates | 415 |
| §5.3 Decomposition Strategy | How the system picks the best breaking question | 447 |
| §6 Context Path Labeling | Named, tagged `ContextPath` records; three-tier tag taxonomy | 486 |
| §6.3 Path Cache | Active nodes matched against known paths → fast re-resolution | 526 |
| §7 Clarification | Single yes/no confirmation when one candidate dominates but confidence is low | 568 |
| §7.5 Goal Tracking | Multi-step goals with `GoalStatus`, revision support, parallel sub-goals | 589 |
| §7.5.4 Urgency Scoring | `urgency:` and `impact:` tags drive priority when sub-goals compete | 645 |

---

## Learning & Memory (§8–§11)

| Section | Purpose | Line |
| ------- | ------- | ---- |
| §8 Learning from Interaction | How confirmed/rejected sessions update the graph | 680 |
| §9 Reinforcement Strategy | Positive/negative weight update formulas; path-level scaling | 708 |
| §10 Latent Node Discovery | Co-occurrence monitoring → auto-created hidden concept nodes | 740 |
| §11 Weak Answer Memory | Uncertain/rejected answers stored in `weak_memory.json` for later correction | 781 |
| §11.3 UI Context Memory | Per-turn record of what was shown, clicked, dismissed — prevents repeated options | 818 |
| §11.5 User Profile | `UserProfile` with `SkillLevel`; drives question de-prioritisation and response verbosity | 855 |

---

## CLI & Storage (§12–§13)

| Section | Purpose | Line |
| ------- | ------- | ---- |
| §12 CLI Behaviour | Interactive loop, single-query mode, `--explain` output format | 919 |
| §13 Memory Layout | File layout: nodes / edges / paths / questions / solutions / weak_memory / sessions / goals | 974 |

---

## Knowledge Base & Growth (§14–§16)

| Section | Purpose | Line |
| ------- | ------- | ---- |
| §14 Initial Knowledge Base | Seed nodes, edges, breaking questions, and paths to start from | 1024 |
| §15 Automatic Context Expansion | Unknown tokens → provisional nodes → promotion after 3 confirmations | 1068 |
| §15.5 Noise Handling | Fuzzy matching (edit distance, n-gram, emotional strip); partial activation; incomplete info tolerance | 1085 |
| §16 Context Bias | High-weight edge dominance; exploration noise ε prevents lock-in | 1159 |

---

## Sizing & Metrics (§17)

| Section | Purpose | Line |
| ------- | ------- | ---- |
| §17 Expected System Size | Component-by-component size budget; total < 45 MB | 1179 |
| §17.5 Outcome Metrics | 8 production metrics (resolution rate, escalation rate, friction score, …); `--metrics` command | 1194 |

---

## Development Phases (§18)

| Phase | Capability | Line |
| ----- | ---------- | ---- |
| 0 | Compilable skeleton, file I/O | 1235 |
| 1 | Static keyword lookup from seed data | 1260 |
| 2 | Graph activation and propagation | 1291 |
| 3 | Single yes/no clarification | 1326 |
| 4 | Multi-branch breaking questions | 1359 |
| 5 | Named path recording with tags | 1395 |
| 6 | Path-level cache (fast re-resolution) | 1433 |
| 7 | Session recording with audit trail | 1465 |
| 8 | Reinforcement learning | 1494 |
| 9 | Weak answer memory | 1527 |
| 10 | Latent node discovery | 1562 |
| 11 | Automatic context expansion | 1599 |
| 12 | Bias tuning and exploration noise | 1635 |
| 13 | BM25 + n-grams + session carry-forward + composite answers | 1689 |

---

## System Comparison & Future Directions (§19–§20)

| Section | Purpose | Line |
| ------- | ------- | ---- |
| §19 System Comparison | Feature table: this system vs. LLM | 1838 |
| §20.1 Neural Embedding | Fuzzy token matching via word2vec/GloVe (~10–30 MB) | 1867 |
| §20.2 Neural Re-ranker | Shallow MLP reorders graph-produced candidates; never overrides the graph | 1895 |
| §20.3 Intent Classifier | Routes query to correct domain persona before activation | 1916 |
| §20.4 Graph Distillation | Distil routing logic into tiny neural fast-path; graph remains authoritative | 1938 |
| §20.5 Persona Graphs | Separable domain knowledge files; inspectable, swappable, composable | 1960 |
| §20.6 NL Answer Formatter | Optional tiny model for natural phrasing; `--raw` disables it | 2021 |
| §20.7 Distributed Sharing | Export / merge persona graphs; weighted-average merge strategy | 2038 |
| §20.8 Network Service | Extract engine as library; add HTTP + Discord/Slack/Teams/Web adapters | 2057 |
| §20.9 Pre-neural ROI | BM25, n-grams, carry-forward, composite answers — ~300 LOC, zero deps | 2143 |
| §20.10 Telecom Vertical Slice | Full connectivity diagnosis agent spec: 4 actions, breaking questions, layer validation table | 2167 |
| §20.11 Event-Driven Core | `EngineInput::SystemEvent`; system initiates turns on outage/device/action events | 2248 |

---

## Summary (§21)

One-paragraph system description. Line 2319.
