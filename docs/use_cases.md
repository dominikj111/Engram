# Engram — Target Use Cases

This document describes the primary deployment contexts for the Engram reasoning kernel.
It is maintained separately from the architecture proposal so use-case framing can evolve
independently of implementation detail.

---

## Strategic Priority

**Primary focus: §8 — LLM Agent Mesh / Cost Optimizer.**
This is the clearest path to revenue and investor attention. The narrative is simple,
the value is CFO-legible ("reduce LLM API spend by 70-80% in bounded domains"), and it
positions Engram *with* the AI ecosystem rather than against it. Switching cost compounds
as the graph accumulates confirmed session knowledge — a moat that grows automatically.

**Long-term target: §7 — Hierarchical Distributed Aggregation.**
The structural differential privacy angle is technically novel and addresses a large
market (privacy-preserving analytics). Build this after §8 generates revenue and
credibility to fund it.

**Deferred: §2 — Industrial Domain Agent.**
Requires deep vertical expertise and long enterprise sales cycles. Strong eventual use
case; wrong first market. Revisit once the core engine is proven.

**All other use cases** are natural extensions that emerge from the same architecture.
None require a strategic detour — they become available as the §8 roadmap phases
are completed.

---

## Roadmap to §8

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

**The problem:** Teams write down knowledge as markdown — runbooks, wikis, confluence
pages. Those documents are static text: the knowledge is implicit in the prose, requires
reading and interpretation to use, and drifts out of date silently. An LLM reading a
markdown runbook hopes to extract the right rule. An Engram graph *is* the rule —
machine-readable, typed, directly executable, and self-correcting from session outcomes.
No interpretation layer. No hallucination surface. No wiki rot.

**What Engram stores instead:** Every session that reaches a confirmed solution reinforces
a reasoning path in the graph. The graph accumulates *what the team collectively confirmed
as correct* — not who said it, not when, not in what tone. The result is a queryable
store of confirmed facts: activate the graph with a context and retrieve the confirmed
path, confidence score, and ruled-out candidates — structured recall, not text search.

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
latent edge — finding shared mental models the team has not explicitly written down.

**Distributed merge (§20.7) enables cross-team knowledge.** Two teams running separate
instances can export and merge graphs — the result encodes the union of both teams'
confirmed experience, with confidence proportional to how many sessions support each path.

**Target contexts:**

- Engineering on-call runbooks distilled from incident resolutions
- Support team shared knowledge base built from ticket outcomes
- DevOps procedure graphs accumulated from deployment sessions
- Any team where the same problems recur and the solutions are deterministic

### 1.1 Incident Post-Mortem Distillation

Post-mortems are written, then rot in Confluence. The valuable artifact is never the
document — it is the resolution path. Feed each confirmed resolution into the graph
instead of a wiki page.

After 20 incidents: `high_latency + cache_miss → CheckDatabaseIndexes [confidence: 0.87, n=20]`

The next on-call engineer gets the distilled experience of two years of incidents, not a
pile of documents to skim at 3am. No one curates a runbook — the graph self-organises
from outcomes. Escalation nodes capture cases that needed human intervention, recording
what was tried first — useful context for the next incident of the same type.

### 1.2 New Engineer Onboarding

"How do I run the test suite locally?" asked by 40 engineers over 3 years. Currently
answered in Slack threads, half of which are wrong by now.

Engram walks each question through breaking questions (which service? which environment?),
reinforces the paths that resolve successfully, and accumulates a graph that encodes what
actually works — not what someone thought was correct when they wrote the wiki page.
The graph improves from every onboarding session. Outdated paths lose weight when they
produce weak-memory entries.

---

## 2. Industrial Domain Agent (Vertical Slice) — *Deferred*

> **Status:** Deprioritised. Requires deep vertical domain expertise and long enterprise
> sales cycles. The architecture supports it fully — revisit once the §8 agent mesh is
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

## 4. Event-Driven Automation and Business Logic

The core pattern: an event arrives, the graph activates, breaking questions narrow the
context if needed, an action contract is selected, the execution layer runs it, the
session closes. This pattern covers both technical automation and business process
decisions — the architecture is identical, only the domain differs.

**Policy engine controls dispatch:**

- Permission level per action (None / Verified / Authenticated / Admin)
- Rate limits prevent accidental loops
- Confirmation requirement before destructive actions
- Rollback availability tracked per action type

**Business logic parity across frontend and backend:** The same graph compiled to WASM
runs in the browser and natively on the server. The same rules gate UI transitions
("can this user proceed to checkout?") without a roundtrip and enforce backend decisions
— eliminating the class of bugs that comes from business logic drift between layers.
Long-running process state lives in the application database; Engram is the rules layer,
not the state store.

**The graph self-improves from outcomes.** Paths that resolve correctly accumulate weight;
paths that produce weak memory entries decay. Process optimisation becomes automatic.

```text
payment_failed event
  → session: context={failure_code, retry_count}
  → breaking question: retry_count < 3?
  → yes → RetryPayment
  → no  → NotifyUser + FlagForReview

DeviceOffline event
  → breaking questions narrow root cause
  → action selected: CheckLineStatus or ScheduleEngineer
  → policy engine validates → execution layer runs
  → outcome feeds back into graph as session
```

### 4.1 CI/CD Test Failure Triage

A test fails in CI. Is it infrastructure, a race condition, or a genuine regression?
An Engram graph walks the space: which suite? which error pattern? previously seen?
intermittent or consistent? The graph learns from how engineers actually resolve
failures — after enough sessions, common signatures route directly to the right fix
path with no human intervention required.

### 4.2 Structured Log and Observability Intelligence

Log events arrive as structured activations — service, error type, frequency pattern.
The graph routes to a candidate cause with a confidence score. Sessions where an
engineer confirms the diagnosis reinforce the path. The graph stores patterns, not log
content — a shared instance encodes "what this error pattern means" without any raw log
data being shared or stored centrally.

```text
service=auth + error=timeout + frequency=burst
  → CheckDatabaseConnectionPool  [confidence: 0.83, n=17]
```

### 4.3 Git Commit and Change Analysis

A commit arrives as an event. Concept nodes activate — schema_change, auth_path,
migration_present. The graph routes to action contracts — RequiresSecurityReview,
NotifySecurityTeam, BlockMergeUntilApproved. Not raw diffs, not author names — the
reasoning path from change signature to action, reinforced each time the pattern repeats.

```text
files=[auth/*, db/migrations/*] + type=schema_change
  → RequiresSecurityReview  [confidence: 0.79, n=23]
```

---

## 5. Compliance and Regulatory Decision Routing

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

## 6. Embedded and Offline Field Diagnostics

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

## 7. Hierarchical Distributed Aggregation

**The core idea:** Local Engram instances (including browser-embedded WASM instances)
emit compact graph deltas — not raw events, not PII — to server-side instances that
merge them upward.

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

**Architectural requirements:**

- WASM compilation target for the reasoning engine
- Compact graph delta serialization format (binary diff of edge weights + new nodes)
- Merge protocol with conflict resolution — see architecture.md §7.2 for the full
  session-count-weighted merge strategy and boundary deduplication rules
- Hierarchical node topology configuration
- Storage backend abstraction (architecture.md §6) — delta streams and remote graph
  sources require a `RestApiStore` or `SocketStore` backend

---

## 8. LLM Pre-Memory, Preprocessor, and Agent Mesh

**The problem with LLM agents today:** Every query hits the model cold. The model reads
the full conversation history, re-reasons from scratch, burns tokens and GPU time. For
bounded domains, most queries are not novel — they are variations on paths the system
has resolved hundreds of times before.

### 8.1 Compressed Memory

The graph *is* the compressed memory passed to the LLM. A structured graph path (a few
hundred bytes) replaces a multi-turn conversation history (thousands of tokens). The LLM
gets more signal with less context window, and never has to re-disambiguate what was
already resolved in previous turns.

### 8.2 Query Preprocessor

A specialist Engram agent does not try to answer the query — it tries to *structure* it.
By the time the query reaches the LLM, the specialist has already:

- resolved which domain applies
- ruled out irrelevant candidates via breaking questions
- extracted and typed the parameters
- established current confidence and what ambiguity remains open

The LLM receives a lean, structured handoff instead of raw conversation. It can skip
straight to the genuinely novel part. This saves tokens, reduces hallucination surface,
and makes the LLM response easier to parse back into the graph.

### 8.3 Specialist Agent Mesh

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

Each specialist graph is owned and updated independently, has locally meaningful
confidence, can be deployed or rolled back without touching other domains, and fails
in isolation.

### 8.4 Cost Profile

```text
Query volume breakdown (mature bounded domain):
  ~70-80%  handled by specialist graph alone      — microseconds, zero API cost
  ~15-25%  specialist preprocesses, LLM completes — reduced tokens, lower latency
  ~5%      genuinely novel, full LLM context      — full cost, but rare

Over time, the LLM teaches the graph and the first bucket grows.
The system is a self-improving cost optimizer.
```

---

## 9. MCP Server — Engram as a Knowledge Database for LLM Agents

Engram as an MCP server inverts the relationship from §8. In §8, Engram escalates to
an LLM when it cannot resolve a query. Here, the **LLM calls Engram as a tool** —
querying the graph mid-reasoning to retrieve structured knowledge the model itself does
not carry.

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
```

### Why this is different from RAG

RAG retrieves text chunks by vector similarity — the LLM gets a paragraph that might
contain the answer. Engram returns a **reasoning path**:

| | RAG | Engram MCP |
| --- | --- | --- |
| What is returned | Text chunks ranked by similarity | Typed path with confidence and ruled-out candidates |
| Why this answer | Unknown — similarity score only | Explicit: N confirmed sessions, these dimensions resolved |
| Staleness signal | None | Edge weight decay — low-confidence paths are flagged |
| Learning from LLM use | None | LLM-confirmed answers reinforce the graph |
| Attribution | Retrieves source documents | Structurally absent — patterns only |

Multiple LLM agents pointing at the same Engram instance share the graph — accumulated
patterns from thousands of sessions — without sharing raw conversations. When an LLM
agent confirms an answer, that outcome feeds back into the graph as a session. The LLM
teaches the graph over time; queries that initially required model reasoning eventually
resolve from Engram alone.

### Architectural requirements

- MCP server wrapper around the Engram query engine — thin adapter, engine unchanged
- `engram.query(text)` tool: returns path, confidence, ruled-out candidates
- `engram.confirm(session_id, outcome)` tool: feeds result back into reinforcement
- `engram.explain(session_id)` tool: returns full reasoning trace for the LLM to cite

---

## 10. Technical Debt Mapping and Refactoring Prioritisation

**The problem:** Refactoring decisions today are made from intuition or blunt static
analysis metrics (cyclomatic complexity, test coverage, churn rate). Neither tells you
*which components actually cause problems* nor *how frequently*. A module can have high
complexity and never be touched in an incident. Another can be clean by every metric and
sit on the critical path of every failure.

**What Engram records:** Each node activated on a confirmed resolution path accumulates
weight proportional to how often it appears in real incidents. After enough sessions:

```text
After 40 resolved sessions over 3 months:

auth_module           weight: 0.89  (on the path in 34 of 40 sessions)
cache_layer           weight: 0.71  (on the path in 22 of 40 sessions)
payment_service       weight: 0.41  (on the path in 12 of 40 sessions)
notification_worker   weight: 0.18  (on the path in 4 of 40 sessions)
```

This is not a static analysis score. It is a ranked list derived from real incidents.
The refactoring priority list writes itself.

**Latent node discovery reveals hidden coupling.** When `auth_module`,
`session_handling`, and `token_refresh` repeatedly co-activate across unrelated
incidents without a documented connection, the graph surfaces a latent node —
a structural problem the team never explicitly named. Static analysis cannot find this:
it requires co-occurrence across runtime problem contexts, not import graphs.

**Negative reinforcement self-corrects the ranking.** A component that appears high on
the list for wrong reasons will naturally decay as the team finds more precise root
causes.

**Target contexts:**

- Refactoring sprint prioritisation ranked by confirmed incident weight, not code metrics
- Detecting hidden coupling that does not appear in import graphs
- Building an evidence base for architectural decisions
- Tracking whether a refactoring actually reduced fault frequency over time

---

## 11. LLM Tool Security Boundary

**The problem with current LLM tool security:** Guardrails today are either system
prompts (bypassable via jailbreak or prompt injection) or runtime checks scattered
across application code (fragile, hard to audit, easy to drift). Neither provides a
structural guarantee — they are fuzzy fuses, not walls.

**What Engram provides instead:** When an LLM calls Engram via MCP, the only operations
available are those explicitly enumerated in `actions.json`. Permissions, rate limits,
and confirmation requirements are declared in `policies.json` and enforced by the
`PolicyEngine` before any execution layer call. The LLM cannot trigger an action that
is not in the contract, cannot bypass a `Permission: Admin` gate, and cannot exceed a
rate limit — not because a prompt says so, but because the execution pathway does not
exist.

```text
LLM tool call: DeleteUser(account_id=42)
  │
  ▼
PolicyEngine evaluates:
  action:               DeleteUser
  required_permission:  Admin
  session_permission:   Authenticated      ← insufficient
  result:               BLOCKED
  graph activation:     permission_denied  → re-authentication breaking question
                                           → or escalation node
```

**This is structural impossibility, not a guardrail.** A guardrail can be bypassed if
the LLM reasons around it. Engram's policy engine sits between the reasoning layer and
the execution layer — the LLM never touches the execution layer directly. The contract
is the interface; the contract is finite; the contract is auditable.

**Properties:**

- **Enumerable action surface** — every operation the LLM can trigger is listed in a
  human-readable file before deployment. Security review is a diff, not a code audit.
- **Centralised policy** — permissions, rate limits, and confirmation requirements are
  in one place (`policies.json`), not scattered across application logic.
- **No prompt dependence** — removing a prompt guardrail changes LLM behaviour. Removing
  an action from `actions.json` makes it structurally unreachable.
- **Auditable by default** — every action attempt, block, and escalation is part of the
  session record. The audit trail is a natural output of the reasoning engine.
- **Runtime policy update** — `policies.json` is evaluated at runtime. A permission rule
  can be tightened without rebuilding or redeploying the graph.

**Comparison to current approaches:**

| Approach | Bypassable | Centralised | Auditable | Runtime update |
| --- | --- | --- | --- | --- |
| System prompt guardrails | Yes — prompt injection | No | No | Yes |
| Hardcoded runtime checks | Difficult — code review | No | Partial | No — redeploy |
| Engram policy engine | No — structural | Yes | Yes | Yes |

**Target contexts:**

- LLM agents with access to sensitive APIs (user data, billing, infrastructure)
- Multi-agent systems where one LLM orchestrates others — policy engine gates every hop
- Regulated environments where every action must be pre-approved and logged
- Any deployment where "the LLM should never be able to do X" must be a guarantee, not a hope

---

## Summary Table

| # | Use Case | Key Benefit | Priority |
| --- | --- | --- | --- |
| 8 | **LLM agent mesh / cost optimizer** | 70-80% of bounded-domain queries handled free; LLM only sees novel cases | **Primary focus** |
| 9 | **MCP server — LLM knowledge database** | Persistent, confidence-weighted, self-improving memory for LLM agents | **Primary focus** |
| 11 | **LLM tool security boundary** | Structural policy enforcement — enumerable actions, centralised permissions, not prompt guardrails | **Primary focus** |
| 7 | Hierarchical distributed aggregation | Structural differential privacy; graph deltas not raw events | Long-term |
| 1 | Team knowledge distillation | Privacy-preserving, attribution-free collective memory; post-mortems and onboarding as natural outputs | Enabled by §8 |
| 4 | Event-driven automation and business logic | Policy-gated action dispatch; CI/CD, log intelligence, and git analysis as sub-cases | Enabled by §8 |
| 5 | Compliance / regulatory routing | Deterministic, auditable decision path — LLM disqualified | Enabled by §8 |
| 10 | Technical debt mapping | Refactoring priorities ranked by confirmed incident weight, not static analysis | Enabled by §8 |
| 3 | Voice assistant runtime | Surface-agnostic `ResponseEnvelope`, numbered-choice breaking questions | Later |
| 6 | Embedded / offline field diagnostics | <1 MB graph on a device, syncs when connectivity returns | Later |
| 2 | Industrial domain agent | Deterministic, auditable, offline-capable vertical deployment | **Deferred** |
