# Engram — Target Use Cases

This document describes the primary deployment contexts for the Engram reasoning kernel.
It is maintained separately from the architecture proposal so use-case framing can evolve
independently of implementation detail.

---

## Strategic Priority

**Primary focus: §15 — LLM Agent Mesh / Cost Optimizer.**
This is the clearest path to revenue and investor attention. The narrative is simple,
the value is CFO-legible ("reduce LLM API spend by 70-80% in bounded domains"), and it
positions Engram *with* the AI ecosystem rather than against it. Switching cost compounds
as the graph accumulates confirmed session knowledge — a moat that grows automatically.

**Long-term target: §14 — Hierarchical Distributed Aggregation.**
The structural differential privacy angle is technically novel and addresses a large
market (privacy-preserving analytics). Build this after §15 generates revenue and
credibility to fund it.

**Deferred: §2 — Industrial Domain Agent.**
Requires deep vertical expertise and long enterprise sales cycles. Strong eventual use
case; wrong first market. Revisit once the core engine is proven.

**All other use cases** are natural extensions that emerge from the same architecture.
None require a strategic detour — they become available as the §15 roadmap phases
are completed.

---

## Roadmap to §15

The critical path from Phase 0 (current) to a demonstrable LLM Agent Mesh:

| Stage | Proposal phases | What it unlocks |
| --- | --- | --- |
| **1. Working reasoning** | Phase 2 (activation), Phase 4 (breaking questions) | Engram can navigate a real domain graph and ask targeted questions |
| **2. Persistent learning** | Phase 5 (path recording), Phase 7 (session recording), Phase 8 (reinforcement) | Sessions reinforce the graph; confidence scores become meaningful |
| **3. One real specialist** | Seed knowledge for one narrow domain (e.g. CI/CD triage or log patterns) | First domain where Engram handles queries without LLM |
| **4. Escalation payload** | New design work — structured handoff context for LLM | Enables the preprocessor story; LLM receives graph path, not raw chat |
| **5. Router agent** | Simple tag-vote classifier (§20.3) | Dispatches to the right specialist; multi-domain mesh becomes possible |
| **6. Measurement** | `--metrics` command (§17.5) | Resolution rate, escalation rate, token savings — the investor demo numbers |

The demonstrable milestone at the end of Stage 3 is: *a query handled entirely by
Engram, with a full reasoning trace, zero API calls, and a measurable confidence score*.
That is the proof of concept. Stages 4–6 turn it into the cost optimizer story.

**Persona graphs (§20.5) are a first-class constraint from Phase 5 onward** — each
specialist must be independently deployable so the mesh architecture is native, not
bolted on later.

---

## 1. Team Knowledge Distillation

**The problem:** Team knowledge lives in chat logs — Slack threads, ticket comments, meeting
notes. It is noisy, personal, and hard to query. Most of it is social glue ("sounds good",
"let me check") wrapping a small core of factual decisions.

**What Engram stores instead:** Every session that reaches a confirmed solution reinforces
a reasoning path in the graph. The graph accumulates *what the team collectively confirmed
as correct* — not who said it, not when, not in what tone.

```text
Raw team chat                     Engram graph after N sessions
─────────────────────────────     ──────────────────────────────
"Alice: try rebooting the         scope=single + duration=intermittent
 switch first"                      → RebootSwitch  [weight: 0.91, n=34]
"Bob: yeah that worked"
"Carol: same fix for me"
"Dave: nope, mine was a
 firmware issue"                  scope=single + duration=permanent
                                    → CheckFirmware [weight: 0.74, n=12]
```

**Privacy is structural, not policy.** Attribution is never recorded. The graph absorbs
the outcome of a conversation, discards the participants. No scrubbing needed.

**Disagreement is handled gracefully.** When team members reach different conclusions on
the same path, weak memory entries accumulate. A path is only promoted to high confidence
after repeated independent confirmation — minority opinions stay provisional rather than
corrupting the shared graph.

**Latent node discovery surfaces hidden consensus.** When multiple people independently
co-activate the same two concepts without a documented connection, the system discovers a
latent edge. This is the engine finding shared mental models the team has not explicitly
written down.

**Distributed merge (§20.7) enables cross-team knowledge.** Two teams running separate
instances can export and merge graphs using the weighted-average strategy — the result
encodes the union of both teams' confirmed experience, with confidence proportional to
how many sessions support each path.

**Target contexts:**

- Engineering on-call runbooks distilled from incident resolutions
- Support team shared knowledge base built from ticket outcomes
- DevOps procedure graphs accumulated from deployment sessions
- Any team where the same problems recur and the solutions are deterministic

---

## 2. Industrial Domain Agent (Vertical Slice) — *Deferred*

> **Status:** Deprioritised. Requires deep vertical domain expertise and long enterprise
> sales cycles. The architecture supports it fully — revisit once the §15 agent mesh is
> proven and a vertical partner opportunity arises.

A bounded, high-stakes domain where determinism and auditability matter more than
conversational fluency. The telecom connectivity diagnosis agent (§20.10) is the
reference implementation.

**Pattern:**

- Domain knowledge is explicit and finite (reboot steps, escalation thresholds, device types)
- Every decision must be traceable (regulatory, SLA, or safety requirements)
- Wrong answers have real consequences — probabilistic LLM output is not acceptable
- The system improves from operator feedback without a retraining cycle

**Other candidate verticals:**

- **Network operations** — fault isolation, circuit provisioning, maintenance windows
- **Industrial maintenance** — equipment fault trees, inspection checklists, part lookup
- **Healthcare triage (non-diagnostic)** — symptom routing, intake categorisation, referral paths
- **Legal intake** — matter classification, document checklist, jurisdiction routing
- **Financial compliance** — product suitability screening, KYC step routing

The graph stores the domain; the policy engine enforces who can trigger which actions;
the execution layer is completely separate. The reasoning kernel never touches external
systems directly.

---

## 3. Voice Assistant Runtime

A voice interface reduces every interaction to: listen → reason → speak → act.
Engram maps cleanly onto that loop.

**How it fits:**

- Breaking questions become numbered choices: *"Say 1 for all devices, say 2 for one device"*
- Action contracts are the command dispatch layer — the voice layer never calls APIs directly
- Event-driven mode (§20.11) lets the system speak first when it detects a relevant event
- `ResponseEnvelope` is surface-agnostic; the voice adapter renders it as TTS

**What Engram adds over a simple intent classifier:**

- Multi-turn goal tracking — the system remembers what it asked two turns ago
- Fallback paths when confidence is low — asks a clarifying question rather than guessing
- Session carry-forward — deferred parameters resolved across turns without re-asking
- Weak memory catches systematic misrecognition patterns and flags them for correction

**Target contexts:**

- Embedded device assistants in industrial or field environments
- Call-centre co-pilot routing to the right resolution path
- Hands-free operator guidance (manufacturing floor, warehouse, field service)

---

## 4. Multi-Command Orchestrator

The reasoning engine selects an action; a validated execution layer runs it. This is the
correct architecture for any system where commands have side effects and must be
authorised before execution.

**Policy engine controls dispatch:**

- Permission level per action (None / Verified / Authenticated / Admin)
- Rate limits prevent accidental loops
- Confirmation requirement before destructive actions
- Rollback availability tracked per action type

**Event-driven entry point (§20.11) enables reactive orchestration:**

```text
External event: DeviceOffline
  → system initiates turn
  → breaking questions narrow root cause
  → action selected: CheckLineStatus or ScheduleEngineer
  → policy engine validates
  → execution layer runs command
  → outcome feeds back into graph as session
```

The graph learns which action sequence resolves which event pattern. Over time,
repeated incident types are handled with higher confidence and fewer breaking questions.

**Target contexts:**

- Infrastructure automation with auditable decision trails
- CI/CD pipeline routing (test failures → triage path → remediation action)
- Scheduled maintenance orchestration
- Incident response runbooks that self-improve from resolution outcomes

---

## 5. Compressed Chat Memory / Knowledge Substrate

Traditional chat archives grow without bound and are expensive to search. Engram offers
an alternative: instead of storing what was said, store what was learned.

**Properties of graph-as-memory:**

| Property | Raw chat archive | Engram graph |
| --- | --- | --- |
| Storage growth | Linear with messages | Sub-linear (edge weight updates, not new rows) |
| Query method | Full-text search | Graph activation (structured) |
| Attribution | Explicit | Absent by design |
| Signal-to-noise | Low (social content dominates) | High (only confirmed outcomes) |
| Staleness handling | Manual deletion | Edge weight decay over time |
| Cross-session synthesis | Manual | Automatic (latent node discovery) |

**What gets compressed in:**

- Confirmed resolution paths (as reinforced edges)
- Rejected paths (as negative weight adjustments + weak memory)
- Emergent concept relationships (as latent nodes)
- User skill patterns (as profile metadata)

**What is intentionally excluded:**

- Who said what
- Timestamps of individual turns
- Verbatim message content
- Social/emotional context

**Reconstruction:** A session can be approximately reconstructed from its recorded
path labels, breaking questions asked, and outcome — enough for audit purposes,
not enough to re-identify individuals.

This makes Engram suitable as a **privacy-preserving collective memory** for teams,
communities, or organisations where the factual substrate matters but individual
attribution does not.

---

## 6. Incident Post-Mortem Distillation

Post-mortems are written, then rot in Confluence. The valuable artifact is never the
document — it is the resolution path. Feed each confirmed resolution into the graph
instead of a wiki page.

After 20 incidents: `high_latency + cache_miss → CheckDatabaseIndexes [confidence: 0.87, n=20]`

The next on-call engineer gets the distilled experience of two years of incidents, not a
pile of documents to skim at 3am. No one curates a runbook — the graph self-organises
from outcomes.

**Architectural fit:** Sessions with `outcome=resolved` directly drive §8 reinforcement.
Escalation nodes capture the cases that needed human escalation, recording what was tried
before the escalation — useful context for the next incident of the same type.

---

## 7. New Engineer Onboarding

"How do I run the test suite locally?" asked by 40 engineers over 3 years. Currently
answered in Slack threads, half of which are wrong by now.

Engram walks each question through breaking questions (which service? which environment?),
reinforces the paths that resolve successfully, and accumulates a graph that encodes what
actually works — not what someone thought was correct when they wrote the wiki page.

**What makes this better than a wiki:** The graph improves from every onboarding session.
Senior engineers stop repeating themselves; the correct path earns higher edge weight with
each confirmation. Outdated paths lose weight when they produce weak-memory entries.

---

## 8. Compliance and Regulatory Decision Routing

"Does this change need a security review?" should always give the same answer given the
same inputs. With an LLM it might not.

A compliance graph encodes the decision tree explicitly: data classification, user-facing
surface, third-party data involved, jurisdiction. Same inputs → same path → same outcome,
always. The full reasoning trace is the audit record.

**Determinism is not a nice-to-have here — it is the requirement.** An LLM is
disqualified on principle. Engram's explicit, reproducible graph navigation is the only
architecture that satisfies this constraint without custom rule-engine tooling.

**Target contexts:** GDPR change classification, SOC 2 control routing, financial product
suitability screening, export control jurisdiction checks.

---

## 9. CI/CD Test Failure Triage

A test fails in CI. Is it infrastructure, a race condition, or a genuine regression?
Currently a human reads the log. A Engram graph walks the space: which suite? which
error pattern? previously seen? intermittent or consistent?

The graph learns from how engineers actually resolve failures. After enough sessions,
common failure signatures route directly to the right fix path with high confidence and
no human intervention required.

**Relationship to logging services:** If Engram is integrated with a structured log or
observability service (see §12), CI failure events become activations on the same graph
that handles production incidents — the same learned paths apply in both contexts.

---

## 10. Embedded and Offline Field Diagnostics

A field technician on a factory floor, no internet, diagnosing a machine fault. An LLM
is out of the question. A narrow-domain Engram graph at <1 MB fits on a device with a
terminal.

Breaking questions walk the technician through the fault tree. The device accumulates
session data locally, synced back to the shared graph when connectivity returns. The
shared graph improves from every field session across the entire fleet of devices.

**The <100 MB constraint and offline-first design make this category of deployment
possible at all** — it is not just a deployment detail, it is a hard differentiator
against every cloud-dependent alternative.

---

## 11. Requirements Disambiguation at the Product/Engineering Boundary

A ticket arrives: "add dark mode support." For which platform? All users or premium?
System-wide or just the dashboard? An LLM interprets this and writes code. Engram asks
breaking questions and records the resolved specification as a labeled path.

The output is a structured, reusable path — not a one-off answer. The same disambiguation
logic accumulates value across every future ticket on the same axes. Over time, common
ambiguity patterns are resolved faster because the breaking questions are already tuned.

---

## 12. Structured Log Intelligence

A personal or team log service generates noisy, growing, hard-to-query data. Engram sits
in front of it as a reasoning layer: instead of full-text searching raw logs, the engineer
navigates a graph — service? error type? frequency pattern? time window? — and arrives at
a candidate cause with supporting evidence.

Sessions where an engineer confirms a log-indicated diagnosis reinforce the path:

```text
service=auth + error=timeout + frequency=burst
  → CheckDatabaseConnectionPool  [confidence: 0.83, n=17]
```

**Privacy benefit for a public log service:** The graph stores patterns, not log content.
A shared Engram instance can encode "what this error pattern means" across many users
without any of their raw log data being shared or stored centrally.

**Architectural note:** Needs a log-event ingestion adapter that maps structured log
fields (service, level, error code, rate) to concept node activations. This is the only
new layer required — the reasoning engine is unchanged.

---

## 13. Git Commit and Change History Analysis

Git history is rich signal: file paths, change types, commit message vocabulary, co-changed
modules. Engram can encode the patterns the team has learned from it.

After enough sessions correlating change patterns with outcomes:

```text
files=[auth/*, db/migrations/*] + type=schema_change
  → RequiresSecurityReview  [confidence: 0.79, n=23]

files=[frontend/components/*] + commit_msg_contains=refactor
  → HighRegressionRisk       [confidence: 0.61, n=9]
```

**What travels into the graph:** Not raw diffs, not author names — just the reasoning
path from change signature to outcome, reinforced each time the pattern repeats.

**Target applications:**

- Code review routing: given the changed files, surface the relevant concerns automatically
- Risk scoring: which commits historically correlate with post-merge bugs?
- Knowledge transfer: new engineers learn which combinations of changes are sensitive

**Architectural note:** Needs a git-event adapter that maps commit metadata and diff
summaries to concept activations. The graph stores *what the team has learned* from git
history, not the history itself.

---

## 14. Hierarchical Distributed Aggregation

**The core idea:** Local Engram instances (including browser-embedded WASM instances)
track user interaction paths and emit compact graph deltas — not raw events, not PII —
to a server-side Engram instance that merges them. Multiple server instances can
aggregate further upward.

```text
Browser (WASM)          Server instance          Global aggregator
──────────────          ───────────────          ─────────────────
user navigates    →     receives delta    →      receives merged
path A→B→C              merges weights           deltas from N
                                                 regional nodes
emits delta:            updated graph:
{A→B: +0.02,            A→B: 0.74
 B→C: +0.03}            B→C: 0.61
```

**What travels between nodes:** Edge weight adjustments, session outcome labels, latent
node discovery signals. Never raw user actions, never identifying information.

**This is structural differential privacy.** The signal is the graph change, not the
event. Usage reports become "this reasoning path is used 10× more than that one" —
actionable product intelligence with no user tracking.

**Practical uses:**

- Product analytics without telemetry pipelines or consent complexity
- A/B testing graph variants: compare edge weights across deployment cohorts
- Progressive knowledge rollout: push a graph update to regional nodes, observe adoption
- Local-first apps that contribute to a global knowledge pool without central data collection

**Architectural requirements this implies:**

- WASM compilation target for the reasoning engine
- Compact graph delta serialization format (binary diff of edge weights + new nodes)
- Merge protocol with conflict resolution (§20.7 covers the strategy, delta format is new)
- Hierarchical node topology configuration

---

## 15. LLM Pre-Memory, Preprocessor, and Agent Mesh

**The problem with LLM agents today:** Every query hits the model cold. The model reads
the full conversation history, re-reasons from scratch, burns tokens and GPU time. For
bounded domains, most queries are not novel — they are variations on paths the system
has resolved hundreds of times before.

### 15.1 Compressed Memory

The graph *is* the compressed memory passed to the LLM. A structured graph path (a few
hundred bytes) replaces a multi-turn conversation history (thousands of tokens). The LLM
gets more signal with less context window, and never has to re-disambiguate what was
already resolved in previous turns.

### 15.2 Query Preprocessor

A specialist Engram agent does not try to answer the query — it tries to *structure* it.
By the time the query reaches the LLM, the specialist has already:

- resolved which domain applies
- ruled out irrelevant candidates via breaking questions
- extracted and typed the parameters
- established current confidence and what ambiguity remains open

The LLM receives a lean, structured handoff instead of raw conversation. It can skip
straight to the genuinely novel part. This saves tokens, reduces hallucination surface,
and makes the LLM response easier to parse back into the graph.

### 15.3 Specialist Agent Mesh

Attempting to cover all domains in one large graph creates a fundamental problem:
**confidence scores lose meaning**. "High confidence" in the auth domain and "high
confidence" in the billing domain are calibrated against completely different session
populations. Cross-domain edge weights interfere. One bad update in one domain ripples
into unrelated paths.

The better architecture is a **fleet of small, specialized graphs** — one per domain —
coordinated by a lightweight router:

```text
Query
  │
  ▼
Router agent (lightweight — domain classification only)
  │
  ├── auth domain    → Auth specialist graph
  ├── billing domain → Billing specialist graph
  ├── infra domain   → Infra specialist graph
  │       │
  │       └── High confidence: answer directly (no LLM)
  │       └── Low confidence: produce structured handoff
  │
  └── ──────────────────────────────────────────────────►
                                                    LLM
                              (receives structured context, not raw conversation)
                                                    │
                                                    ▼
                                    Result feeds back into specialist graph
```

Each specialist graph:

- is owned and updated independently by the team responsible for that domain
- has locally meaningful confidence (calibrated only against its own sessions)
- can be deployed, rolled back, or replaced without touching any other domain
- fails in isolation — a broken billing graph does not affect auth

The router itself can be a tiny Engram graph: narrow, fast, and self-improving as
routing decisions are confirmed or corrected.

**This is §20.5 Persona Graphs as the core architecture, not a future direction.**
The agent mesh is how persona graphs compose at runtime. See the note in §20.5 of
the proposal for the elevation rationale.

### 15.4 Architectural Requirements

- Escalation payload must carry structured graph context (path labels, ruled-out
  candidates, confidence history), not just free text
- LLM response parser that maps the answer back to a graph path for reinforcement
- Router agent: small domain-classification graph or tag-vote classifier (§20.3)
- Persona graph format: versioned, signable, independently deployable (§20.5)
- Optional: LLM-generated breaking questions feed into the graph as new nodes
  (§15 automatic context expansion already covers promotion after confirmation)

### 15.5 Cost Profile

```text
Query volume breakdown (mature bounded domain):
  ~70-80%  handled by specialist graph alone      — microseconds, zero API cost
  ~15-25%  specialist preprocesses, LLM completes — reduced tokens, lower latency
  ~5%      genuinely novel, full LLM context      — full cost, but rare

Over time, the LLM teaches the graph and the first bucket grows.
The system is a self-improving cost optimizer.
```

---

## 16. MCP Server — Engram as a Knowledge Database for LLM Agents

Today's LLMs carry memory in two ways: the context window (resets every
conversation) and file-based storage (flat, unstructured, manually managed).
Neither accumulates knowledge across sessions. Neither learns from confirmed
outcomes. Neither provides structured, confidence-weighted recall.

Engram as an MCP server inverts the relationship from §15. In §15, Engram
escalates to an LLM when it cannot resolve a query. Here, the **LLM calls
Engram as a tool** — querying the graph mid-reasoning to retrieve structured
knowledge the model itself does not carry.

```text
LLM reasoning step
  │
  └── tool call: engram.query("auth timeout pattern")
        │
        ▼
        returns: {
          path: "error=timeout + service=auth → CheckConnectionPool",
          confidence: 0.91,
          confirmed_n: 34,
          ruled_out: ["ScaleReplicas (confidence: 0.31)"],
          breaking_questions_resolved: ["scope_dimension", "load_dimension"]
        }
        │
        ▼
  LLM incorporates structured context — typed result, not a raw string
```

### Why this is different from RAG

RAG retrieves text chunks by vector similarity — the LLM gets a paragraph
that might contain the answer. Engram returns a **reasoning path**:

| | RAG | Engram MCP |
| --- | --- | --- |
| What is returned | Text chunks ranked by similarity | Typed path with confidence and ruled-out candidates |
| Why this answer | Unknown — similarity score only | Explicit: N confirmed sessions, these dimensions resolved |
| Staleness signal | None | Edge weight decay — low-confidence paths are flagged |
| Learning from LLM use | None | LLM-confirmed answers reinforce the graph |
| Attribution | Retrieves source documents | Structurally absent — patterns only |

### Shared memory across LLM agents

Multiple LLM agents pointing at the same Engram instance share the graph —
accumulated patterns from thousands of sessions — without sharing raw
conversations. Each agent gets the benefit of every session any agent
contributed to. The structural privacy property holds: agents share
*what was confirmed as correct*, never *who said what*.

This is the missing layer in the current LLM memory stack:

```text
Current LLM memory:
  context window     — resets each conversation
  file storage       — flat, unstructured, manually curated

With Engram MCP:
  context window     — resets each conversation
  Engram graph       — persistent, structured, self-improving, confidence-weighted
  file storage       — raw content when needed
```

### The graph learns from LLM use

When an LLM agent calls Engram, reasons with the result, and the user
confirms the answer — that outcome feeds back into the graph as a session,
reinforcing the path. The LLM teaches the graph over time. Queries that
initially required LLM reasoning eventually resolve from the graph alone.

### Architectural requirements

- MCP server wrapper around the Engram query engine — thin adapter, engine unchanged
- `engram.query(text)` tool: returns path, confidence, ruled-out candidates
- `engram.confirm(session_id, outcome)` tool: feeds result back into reinforcement
- `engram.explain(session_id)` tool: returns full reasoning trace for the LLM to cite
- Multiple Engram instances (specialist graphs) exposed as separate MCP tools,
  allowing the LLM to route to the right domain knowledge explicitly

---

## Summary Table

| # | Use Case | Key Benefit | Priority | Relevant Sections |
| --- | --- | --- | --- | --- |
| 15 | **LLM agent mesh / cost optimizer** | 70-80% of bounded-domain queries handled free; LLM only sees novel cases | **Primary focus** | §3.4, §20.1–§20.5 |
| 16 | **MCP server — LLM knowledge database** | Persistent, confidence-weighted, self-improving memory for LLM agents via MCP | **Primary focus** | §8–§11, §20.8 |
| 14 | Hierarchical distributed aggregation | Structural differential privacy; graph deltas not raw events | Long-term | §20.7 |
| 6 | Incident post-mortem distillation | Self-organising runbook from confirmed resolutions | Enabled by §15 | §8, §9 |
| 7 | New engineer onboarding | Self-improving knowledge base that outlasts any wiki | Enabled by §15 | §8, §11 |
| 9 | CI/CD test failure triage | Graph learns which failure patterns map to which root causes | Enabled by §15 | §8, §20.11 |
| 12 | Structured log intelligence | Reasoning layer over logs; graph stores patterns not content | Enabled by §15 | §4, §8 |
| 13 | Git history pattern analysis | Change signatures → risk/routing paths, no raw diffs stored | Enabled by §15 | §8, §15 |
| 1 | Team knowledge distillation | Privacy-preserving, attribution-free collective memory | Enabled by §15 | §8–§11, §20.7 |
| 5 | Compressed chat memory | Sub-linear growth, high signal-to-noise, no attribution | Enabled by §15 | §8–§11, §13 |
| 8 | Compliance / regulatory routing | Deterministic, auditable decision path — LLM disqualified | Enabled by §15 | §3.6, §4.5 |
| 11 | Requirements disambiguation | Structured, reusable paths replace one-off PM/eng conversations | Enabled by §15 | §5, §7.5 |
| 3 | Voice assistant runtime | Surface-agnostic `ResponseEnvelope`, numbered-choice breaking questions | Later | §20.8, §3.5 |
| 4 | Multi-command orchestrator | Policy-gated action dispatch, event-driven initiation | Later | §3.6, §20.11 |
| 10 | Embedded / offline field diagnostics | <1 MB graph on a device, syncs when connectivity returns | Later | §17, §20.5 |
| 2 | Industrial domain agent | Deterministic, auditable, offline-capable vertical deployment | **Deferred** | §20.10, §3.6 |
