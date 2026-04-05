# Engram — System Comparison, Future Directions, and Summary

*Part of the [Engram design documentation](proposal.md).*

---

## 19. System Comparison

| Feature                   | This System                     | LLM                   |
| ------------------------- | ------------------------------- | --------------------- |
| Compute requirement       | Tiny (<50 MB RAM)               | Huge (GB+)            |
| Learning method           | Incremental                     | Full retraining       |
| Explainability            | Full path trace                 | Limited               |
| Deterministic             | Yes                             | No                    |
| Breaking question logic   | Explicit, labeled               | Implicit              |
| Path reuse / caching      | Named paths                     | None                  |
| Latent concept discovery  | Automatic                       | Baked in              |
| Offline / air-gapped      | Yes                             | Rarely                |
| Action contracts          | Typed, enumerable, auditable    | Implicit / unchecked  |
| Goal tracking             | Multi-step, revisable           | Single-turn           |
| User profile              | Statistical, transparent        | None / opaque         |
| Escalation                | Structured handoff with context | "I don't know"        |
| Domain isolation          | Separate persona files          | Entangled weights     |
| System actions            | Execution layer separated       | Direct / unvalidated  |

---

## 20. Future Directions

These are not planned phases. They are architectural directions worth
considering once the core 12-phase system is stable and producing session
data. Each one is independent — none requires the others.

---

### 20.1 Neural Embedding Layer for Fuzzy Token Matching

The current tokenizer does exact label matching with a hand-written alias map.
This breaks on synonyms, paraphrasing, typos, and domain drift.

A small **static word embedding model** (word2vec or GloVe scale, ~10–30 MB)
would replace exact matching with nearest-neighbor lookup in vector space:

```text
Input token: "borrowing"
Nearest nodes by cosine similarity:
  borrow_checker   0.91
  ownership        0.74
  reference        0.68
```

The graph engine stays unchanged. The embedding layer is a preprocessing
step only — it maps input tokens to node activation seeds. The reasoning
is still entirely graph-based and deterministic once seeds are chosen.

This single addition would dramatically improve recall on natural language
queries without touching any of the graph logic.

**Size budget:** a trimmed embedding vocabulary covering the node label set
can be as small as 5–15 MB. Well within the 100 MB constraint.

---

### 20.2 Small Neural Candidate Re-ranker

After activation propagation, the graph produces a ranked list of candidate
solutions with scalar scores. A very small neural re-ranker (a shallow MLP,
~100K parameters) could take as input:

- the top-K candidate activation scores
- the query token vector (averaged embeddings)
- the session's active tag set (one-hot encoded)

and output a refined ranking. This is trained continuously on confirmed
session outcomes: when a user confirms solution X, that (query, X) pair
is a positive training example.

The key property: the re-ranker **never overrides the graph**. It only
reorders candidates that the graph already surfaced. If the graph says a
node is unreachable, the re-ranker never sees it. Explainability is
preserved because the graph still determined the candidate set.

---

### 20.3 Intent and Domain Classifier

Before graph activation begins, a tiny classifier could route the query to
the correct entry domain:

```text
Input: "why does my goroutine hang"
Classifier output:
  domain: golang        0.87
  domain: concurrency   0.76
  intent: troubleshoot  0.94
```

This activates the right persona graph (see §20.5) immediately rather than
letting cross-domain noise propagate through activation.

Training data comes naturally from session history: every confirmed session
is a labeled (query, domain) pair. The classifier trains incrementally on
session data, exactly like the graph itself.

---

### 20.4 Graph-to-Neural Distillation

After the graph accumulates substantial session data, a useful optimization
is to **distill the routing logic into a tiny neural network**:

1. Generate synthetic (query, confirmed\_path) pairs from session history
2. Train a small classifier to map query embeddings to path labels
3. Use the neural model as a **fast-path cache** — if it predicts a path
   with high confidence, return it immediately
4. Fall through to full graph propagation on low confidence or cache miss

The graph remains the **authoritative source of truth**. The distilled model
is only an inference accelerator. All learning and reinforcement still happen
on the graph. The neural model is regenerated periodically from updated
session data.

This is the reverse of how LLMs are often used: instead of a neural model
that sometimes explains its reasoning, here the explanation is primary and
the neural shortcut is derivative.

---

### 20.5 Persona Graphs — Separable Domain Knowledge

> **Architectural status:** Originally listed as a future direction. Elevated to
> **core architectural pattern** because it is load-bearing for the LLM pre-memory
> and agent mesh use case (see [use_cases.md §15](use_cases.md#15-llm-pre-memory-preprocessor-and-agent-mesh)).
> A single monolithic graph cannot serve that use case correctly — confidence scores
> lose domain-local meaning, cross-domain edge weights interfere, and independent
> team ownership becomes impossible. The persona graph fleet *is* the architecture
> for multi-domain deployments. Design decisions from Phase 5 onward should treat
> this as a first-class constraint.

This is the most architecturally significant direction.

**The LLM problem:** an LLM contains all domain knowledge, all problem patterns,
all reasoning styles, and all response personas entangled together in one
weight matrix. You cannot inspect, replace, or share just the "Rust expert"
part. Personas in LLMs are implicit — the model selects one based on context
signals it learned during training, in a process Anthropic describes in their
[persona selection model research](https://www.anthropic.com/research/persona-selection-model)
as the AI enacting a character within the implicit space of all characters it
learned to simulate during pretraining.

**The graph alternative:** in this system, a "persona" is simply a
**named, self-contained context graph file**. The base system loads a shared
structural core; persona graphs are layered on top.

```text
knowledge/
  core/
    nodes.json          — universal concepts (error, function, type, loop…)
    edges.json
    questions.json
  personas/
    rust_systems.kg     — Rust ownership, lifetimes, async, cargo…
    python_data.kg      — Pandas, numpy, type hints, asyncio…
    sql_query.kg        — joins, indexes, transactions, normalization…
    concurrency.kg      — threads, locks, channels, deadlock patterns…
```

The persona selection step becomes **explicit and deterministic**: a small
domain classifier (§20.3) or even a simple tag-vote over the active tokens
selects which persona graphs to activate for the current query. No hidden
inference — the selection is logged and inspectable.

**Properties this gives you that LLMs cannot:**

| Property              | LLM personas               | Graph personas                   |
| --------------------- | -------------------------- | -------------------------------- |
| Inspectable           | No — baked into weights    | Yes — plain JSON/binary files    |
| Replaceable           | No — requires retraining   | Yes — swap or patch the file     |
| Composable            | Rarely — blending is noisy | Yes — load multiple simultaneously|
| Distributable         | Impractical at model scale | Yes — a persona is a small file  |
| Conflict-free         | No — personas bleed into each other | Yes — separate namespaces |
| User-contributed      | No                         | Yes — community persona packs    |

**Binary format:** a compiled persona graph is a deterministically serialized
binary (e.g. MessagePack or FlatBuffers) containing nodes, edges, paths, and
questions for one domain. Versioned and signable. An organization could ship
a `company_internal.kg` persona encoding their specific infrastructure
knowledge — something completely impossible with a shared LLM.

**The conceptual inversion:** where an LLM stores knowledge in weights and
selects a persona implicitly, this system stores personas explicitly and
selects them with a transparent routing rule. The reasoning architecture
proposed here is in some ways a structural implementation of what the
Anthropic persona selection model describes as an emergent property of LLMs
— but made explicit, auditable, and composable by design.

#### 20.5.1 The Swarm as Sparse Mixture of Experts

A fleet of specialist Engram graphs coordinated by a router agent is
structurally equivalent to a **sparse Mixture of Experts** architecture —
the same design used inside GPT-4, Mixtral, and most frontier LLMs:

```text
Neural sparse MoE:                  Engram specialist swarm:
──────────────────                  ─────────────────────────
Router selects k of N experts       Router agent selects specialist graph(s)
Only selected experts activate      Only matched specialist processes query
Total parameters: very large        Total knowledge: large (across all graphs)
Compute per token: small (k/N)      Compute per query: small (1-2 specialists)
Routing: learned, opaque            Routing: deterministic, inspectable
Expert weight updates: backprop     Expert updates: edge reinforcement
Expert semantics: unknown           Expert semantics: explicit domain labels
```

The resource efficiency argument is identical: the system carries a large total
knowledge base but activates only the relevant slice per query. The fundamental
difference is that Engram's experts are **explicit, named, independently
deployable graphs** rather than anonymous weight matrices — they can be inspected,
replaced, versioned, and owned by different teams.

**Recursive composition:** A Engram node operating as a pure router runs the same
activation propagation over a *routing graph* (where nodes are specialist agents
rather than domain concepts) and emits a dispatch decision. The routing graph learns
which specialist resolves which query signature via the same reinforcement mechanism
as any other graph. This is a clean recursive structure: the same engine, at every
layer of the hierarchy. Depth is determined by domain complexity, not architectural
constraint.

**The LLM crossover point:** As the specialist count N grows and each graph
accumulates session data, the swarm converges toward LLM-level coverage *for its
known domains*. It never closes the gap on genuinely novel cross-domain reasoning —
that boundary defines when escalation to an LLM is appropriate. The swarm and the
LLM are complementary, not competing: the swarm handles the well-trodden paths
cheaply and deterministically; the LLM handles the novel cases and teaches the swarm
what to encode next.

See [use_cases.md §15](use_cases.md#15-llm-pre-memory-preprocessor-and-agent-mesh)
for the deployment implications and cost profile.

---

### 20.6 Natural Language Answer Formatter (Optional)

Currently, answers are stored template strings on solution nodes. A tiny
conditional text generation model (~5–20 MB, distilled from a larger model)
could produce more natural phrasing given:

- the solution text
- the query tokens
- the active path label and tags

This is purely cosmetic — it styles the delivery, not the content. The
solution node still determines what is communicated. The formatter can be
disabled entirely with `--raw` to return the template string, preserving
full determinism for scripting use cases.

---

### 20.7 Distributed Knowledge Sharing

Once persona graphs are self-contained files (§20.5), the natural extension
is a **knowledge exchange protocol**: users or teams can export their
persona graphs, share them, and merge them.

Merge strategy: when two graphs share a node label, their edges are combined
with weighted averaging:

$$w_{\text{merged}} = \frac{w_A \cdot n_A + w_B \cdot n_B}{n_A + n_B}$$

where $n_A$, $n_B$ are the usage counts from each source graph. Edges
unique to one graph are included at half weight pending local confirmation.

This allows a community to collaboratively build and refine a shared domain
knowledge base while each user's local graph remains private and authoritative.

---

### 20.8 Deployment as a Network Service — Web, Discord, Slack, Teams

The CLI is the development and testing surface. The graph engine itself has
no CLI dependency — it is a pure function:

```text
query + session_state  →  (answer | breaking_question) + updated_session_state
```

This maps directly onto a request/response API. Promotion to a network
service requires a thin layer, not a redesign.

**Architectural path:**

```text
Phase 0–12:  CLI binary
                │
                ▼
Step A:  Extract engine as a library crate (no I/O, pure logic)
                │
                ▼
Step B:  Add HTTP server (axum or actix-web) with two endpoints:
           POST /query        — submit a question, get answer or question
           POST /feedback     — confirm or reject the last answer
                │
                ▼
Step C:  Add connector adapters:
           Discord bot        — maps message events to /query, DMs for feedback
           Slack app          — slash command or mention → /query
           Teams bot          — Adaptive Card responses for breaking questions
           Web chat widget    — WebSocket for streaming the question/answer turn
```

Each adapter is ~100–300 lines. The graph engine underneath is unchanged.

**Why this architecture is fast:**

The graph engine does no I/O during a query — it operates entirely in
memory. Activation propagation over a 5,000-node graph takes microseconds.
Latency is dominated by the network round trip, not by any reasoning step.
No GPU, no model server, no warm-up time. A single modest server can handle
thousands of concurrent sessions.

**Learning from chat interactions:**

Every confirmed answer in Discord or Slack is a reinforcement signal,
identical to a CLI `[y]` confirmation. Every ignored or corrected answer
feeds the weak memory system. A community chat channel becomes a continuous
training signal:

```text
User in #rust-help: "borrow checker error on line 42"
Bot:  "Are multiple parts of the code mutably borrowing the same value?"
User: "yes exactly"
→ rust_ownership_violation confirmed
→ edge weights reinforced
→ session logged
```

The graph improves in real time while users interact naturally. No batch
retraining. No deployment cycle. The same binary that answered the question
is already updated by the time the next question arrives.

**Multi-tenant persona isolation:**

Different channels or servers can load different persona graphs (§20.5):

```text
#rust-help     →  rust_systems.kg
#python-data   →  python_data.kg
#db-questions  →  sql_query.kg
```

The core engine is shared; the knowledge is scoped. A company could run one
service instance serving multiple internal teams, each with their own
domain graph file, without any knowledge bleed between them.

**Deployment size:**

A compiled Rust binary with the graph engine + HTTP server is likely
5–15 MB. The knowledge files add another 10–30 MB. The entire deployment
artifact fits in a Docker image under 100 MB — smaller than most Node.js
`node_modules` directories.

---

### 20.9 High-ROI Enhancements Before Neural Layers

Before introducing any neural component (§20.1–20.4), four purely
algorithmic additions close the majority of the perceptible gap between
this system and an LLM on bounded technical queries. These are formalized
in **Phase 13** of the development plan.

| Addition              | Cost      | Gap it closes                                       |
| --------------------- | --------- | --------------------------------------------------- |
| BM25 retrieval        | ~150 LOC  | Natural language queries that miss exact tokens     |
| N-gram matching       | ~20 LOC   | Multi-word concept labels (`borrow checker` etc.)   |
| Session carry-forward | ~50 LOC   | Multi-turn coherence; avoids re-asking settled dims |
| Composite answers     | ~80 LOC   | Nuanced dual-candidate output instead of binary Q&A |

Total implementation cost: ~300 lines of code, zero new dependencies,
no changes to graph data structures. Together they represent the highest
return on investment before the system crosses into hybrid neural territory.

The recommended insertion point is after Phase 12 (bias tuning) but before
any neural work from §20.1 onward — the system should be fully stable and
generating rich session data before embeddings or classifiers are added.

---

### 20.10 Vertical Deployment Slice — A Telecom Example

The cleanest way to validate the full architecture before building a general
system is to implement one narrow vertical slice end to end: all layers
present, all constraints real, scope deliberately small.

#### Recommended first slice: Internet connectivity diagnosis agent

This is the ideal test case. It is bounded (3–5 actions, 2–3 breaking
question dimensions), high-value (most common telecom support query),
and exercises every architectural layer simultaneously.

#### Actions defined for this slice

```text
action: CheckOutageStatus
  input:  { postcode: String }
  output: { outage: bool, eta_minutes: Option<u32> }

action: CheckLineStatus
  input:  { account_id: String }
  output: { sync_status: String, signal_dbm: f32, error_count: u32 }

action: RebootRouter
  input:  { device_id: String }
  output: { initiated: bool }

action: ScheduleEngineer
  input:  { account_id: String, preferred_date: String }
  output: { booking_ref: String }
```

#### Graph nodes for this slice

```text
connectivity_issue  [Concept]
  → outage_detected          [Solution → Action: CheckOutageStatus]
  → line_fault               [Solution → Action: CheckLineStatus]
  → router_malfunction       [Solution → Action: RebootRouter]
  → engineer_required        [Solution → Action: ScheduleEngineer → Escalation]
```

#### Breaking questions

```text
scope_dimension:
  prompt: "Does the problem affect all devices on your network, or just one?"
  branches:
    all_devices   → line_fault | outage_detected  [path: network_wide_fault]
    single_device → router_malfunction             [path: device_fault]

duration_dimension:
  prompt: "Has the connection been dropping intermittently or completely absent?"
  branches:
    intermittent  → line_fault          [path: intermittent_fault]
    absent        → outage_detected     [path: complete_outage]
```

#### What this slice validates

| Layer                 | What is exercised                                    |
| --------------------- | ---------------------------------------------------- |
| Graph reasoning       | Activation, propagation, breaking questions          |
| Action contracts      | Parameter collection, execution layer separation     |
| Goal tracking         | Multi-turn diagnosis with context carry-forward      |
| Escalation            | engineer_required → structured handoff               |
| User profile          | Novice sees guided steps; Expert sees direct actions |
| Session + weak memory | Failed diagnoses feed correction loop                |
| Observability         | Full trace: every action, branch, outcome logged     |

#### Why this slice, not a general system

Building the general system first means deferring every hard integration
question. This slice forces the action contract boundary, the execution
layer, real backend calls, and the escalation path to be real and working
before any scope is added. Everything learned here transfers directly to
billing, account management, and device provisioning domains — each of
which is just a different persona graph loaded over the same engine.

---

### 20.11 Event-Driven Core

The current system is entirely **query-driven**: the user sends a message,
the engine responds, the session waits. This is correct for a CLI tool and
sufficient for Phase 0–13. But a production service intelligence layer must
also be **event-aware** — able to initiate a turn in response to a system
event, not just a user message.

The architectural shift is small: instead of a single entry point
`(query, session) → response`, the engine gains a second entry point:
`(event, session) → response`.

```rust
enum EngineInput {
    UserQuery(String),
    SystemEvent(Event),
}

struct Event {
    kind:    EventKind,
    payload: HashMap<String, String>,
    time:    Timestamp,
}

enum EventKind {
    OutageDetected,       // network monitoring detected a fault in user's area
    ServiceRestored,      // outage cleared
    BillGenerated,        // new bill available
    ContractExpiring,     // contract end date approaching
    DeviceOffline,        // router stopped phoning home
    ActionCompleted,      // async backend action finished
    ActionFailed,         // async backend action failed
}
```

When an event arrives, the engine activates the corresponding entry node
in the graph exactly as if the user had typed it — the same breaking
questions, action contracts, and escalation paths apply. The difference is
that the system initiates the conversation rather than waiting.

Examples of event-driven behaviour:

```text
Event: DeviceOffline { device_id: "RTR-0042", account_id: "ACC-001" }
→ activate: router_malfunction node
→ confidence: High (event source is authoritative)
→ ResponseEnvelope: "Your router has gone offline. Shall I run a remote
                     reboot?" [Reboot / No thanks / Escalate]

Event: ServiceRestored { postcode: "BT1", outage_id: "OUT-99" }
→ locate open sessions with path label "outage_detected" in same area
→ for each: update goal to Resolved, send notification
→ no questions needed — the event is the answer

Event: ActionCompleted { action: "ScheduleEngineer", booking_ref: "ENG-42" }
→ close the deferred ActionPending session
→ confirm goal Resolved with booking reference
```

**Why this matters:** the difference between a chatbot and a service
intelligence layer is exactly this. A chatbot waits to be asked. A service
layer knows what is happening and speaks first — but only when it has
something real and actionable to say, sourced from the same typed action
contracts and policy engine that govern every user-initiated flow.

Events are persisted to an `events.json` log. Unprocessed events (e.g.
the user's session was closed before the async action completed) are queued
and delivered at the start of the next session.

---

## 21. Summary

This system is not a chatbot in the conventional sense.  
It is a **self-organizing semantic reasoning engine** built on:

- a **weighted context graph** that learns from every interaction
- **breaking questions** that decompose ambiguous paths into labeled branches
- **context path labeling** that names, tags, and reuses known reasoning routes
- **latent node discovery** that surfaces hidden structure automatically
- **weak memory** that converts mistakes into future knowledge

It occupies the lineage of expert systems and semantic networks, extended with
modern reinforcement ideas — remaining compact, fully transparent, and
continuously self-improving.
