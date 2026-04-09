# Engram — Architecture and Data Structures

*Part of the [Engram design documentation](proposal.md).*

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

### 2.1 The Graph as a Problem Space Map

Engram does not store a single answer per problem. It stores an evolving map
of the full problem space — all known causes for a given symptom, each with
its own confidence score calibrated by how often it turned out to be the real
cause across all sessions.

The same symptom ("connection timeout") may have many valid root causes:
undersized connection pool, missing index, lock contention, network issue,
firewall rule. Each is a real path in the graph. Over time, the graph
explores all of them and ranks them by confirmed frequency:

```text
timeout
  → connection_pool      [weight: 0.76, confidence: 0.71, n=28]  ← common
  → missing_index        [weight: 0.63, confidence: 0.58, n=14]
  → lock_contention      [weight: 0.51, confidence: 0.62, n=11]
  → network_issue        [weight: 0.34, confidence: 0.45, n=6]
  → firewall_rule        [weight: 0.18, confidence: 0.40, n=3]   ← rare
```

**Decay does not delete — it deprioritises.** A path with low weight is still
traversable when activation is high enough. No door is permanently closed.
If a rare root cause becomes more common (a new firewall policy, a schema
change), its sessions push its weight back up naturally.

**Early sessions are exploratory — mature sessions are efficient.** The first
time a novel problem is encountered, the path is unknown and the session may
be messy: failed attempts, reverts, escalation. Each failed attempt records
negative reinforcement. Once a working path is confirmed, it earns weight.
By the tenth similar session, the graph routes directly to the likely cause —
skipping dead ends that earlier sessions already paid the cost to explore.

This is what makes Engram useful at scale: not that it knows the answer
immediately, but that it accumulates the collective diagnostic experience of
every session that came before.

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

## 2.6 The Input Boundary: Text In, Node IDs Out

User input text crosses the system boundary exactly once: at the tokeniser
(§4.1). The tokeniser maps tokens to node IDs and discards the text. Nothing
downstream of the tokeniser holds user-originated strings.

```text
User input text
      │
      ▼  tokenise + normalise
Node ID activation vector   ← text discarded here
      │
      ▼
Graph navigation, reinforcement, storage — all operate on node IDs only
```

**Two very different cases cross this boundary:**

**Hard case — human freetext.** The user types natural language. The tokeniser
must map vocabulary it was never explicitly taught: synonyms, paraphrasing,
typos, domain drift. `"dying under load"` must activate `[timeout, high_concurrency]`.
This is the central NLP challenge for any keyword-based system. Tokeniser quality
directly determines recall — a missed mapping means the wrong path (or no path) is
activated, regardless of how good the graph is. The quality spectrum from alias map
through BM25 to embeddings is documented in §4.1; each tier is a drop-in replacement
for seed generation without changing the graph engine.

**Easy case — MCP and agent callers.** When Engram is called via MCP, the LLM emits
a structured tool invocation — the parameters are already named concept tokens:

```text
// Human freetext — tokeniser must map "dying under load" → [timeout, high_concurrency]
"my orders endpoint keeps dying under load"

// MCP tool call — activation vector arrives pre-formed
engram.query({ nodes: ["timeout", "database", "orders"] })
```

In the MCP case, the `nodes` array is the activation vector. The tokeniser
becomes a trivial ID lookup rather than an NLP problem. The LLM has already
performed semantic extraction; Engram receives the result of that work, not
the raw text. This is why MCP is the natural deployment surface for Engram
in agent pipelines: the protocol already speaks the graph's vocabulary, with
no semantic bridge required.

This is not a sanitisation policy. Sanitisation assumes sensitive content
might slip through and tries to catch it. This is a structural guarantee:
there is no pathway by which input text reaches any storage layer, because
the interface between input processing and the rest of the system is typed
as node IDs, not strings.

**What is stored per session:**

- Node IDs activated
- Path labels traversed (curated names from the knowledge graph, not user words)
- Breaking question node IDs asked
- Session outcome

**What is never stored:**

- Input text in any form
- User responses verbatim
- Tokens derived from input
- Free-text descriptions of what the user said

**Curated text** — solution node bodies, breaking question prompts, node labels,
path names — is authored by the knowledge graph maintainer and is not
user-originated. It does not fall under this constraint.

---

## 3. Knowledge Representation

### 3.1 Context Graph

The knowledge base is a **weighted directed graph**. Nodes are concepts;
edges are contextual transitions.

```text
compile_error
   → borrow_checker         (weight: 0.78, confidence: 0.62)
      → mutable_reference_conflict
      → lifetime_mismatch
```

### 3.2 Node Structure

```rust
struct Node {
    id:           u32,
    label:        String,       // e.g. "borrow_checker"
    kind:         NodeKind,     // Concept | Question | Solution | Latent | Escalation
    activation:   f32,          // transient score during query processing
    tags:         Vec<String>,  // semantic tags, e.g. ["ownership", "rust"]
}
```

Node kinds:

| Kind           | Description                                                                  |
| -------------- | ---------------------------------------------------------------------------- |
| `Concept`      | A domain term or context anchor                                              |
| `Question`     | A breaking or clarifying question node                                       |
| `Solution`     | A leaf node with a text answer or typed action contract                      |
| `Latent`       | Auto-discovered hidden concept                                               |
| `Escalation`   | Path terminus that exports structured context for handoff — planned Phase 12 |

### 3.3 Edge Structure

```rust
struct Edge {
    src:         u32,
    dst:         u32,
    weight:      f32,      // [0.0, 1.0] — path strength
    confidence:  f32,      // [0.0, 1.0] — reliability estimate
    usage_count: u32,
    path_labels: Vec<String>,  // path tags this edge belongs to
}
```

Example:

```text
borrow_checker → mutable_reference_conflict
  weight:      0.81
  confidence:  0.75
  usage_count: 14
  path_labels: ["ownership_violation", "single_threaded"]
```

### 3.4 Solution Node Variants

A `Solution` node carries either a text answer or a typed **action contract**.
The graph selects which solution to activate; a separate execution layer is
responsible for running actions. The reasoning engine never calls external
APIs directly — this boundary is non-negotiable for production deployments.

```rust
enum SolutionPayload {
    Text(String),
    Action {
        name:   String,           // e.g. "CheckLineStatus"
        params: Vec<ActionParam>, // typed parameter descriptors
    },
    Escalation {
        reason:  String,          // why escalation was triggered
        context: EscalationContext, // structured session state for handoff
    },
}

struct ActionParam {
    key:        String,            // e.g. "account_id"
    kind:       ParamKind,         // String | Integer | Boolean | Enum(Vec<String>)
    required:   bool,
    resolution: ResolutionChain,   // ordered resolution steps (see below)
}

struct ResolutionChain {
    steps: Vec<ParamSource>,       // tried in order; first success wins
}

enum ParamSource {
    SessionContext,      // already known from current session
    ConversationInfer,   // extract from recent user messages (e.g. postcode in "BT1 down")
    BackendPrefill,      // query a backend lookup before asking the user
    UserInput,           // only reached if all prior steps fail
}
```

**Parameter resolution pipeline** — for each required parameter, the system
tries each step in the `ResolutionChain` in order. The user is asked only
for parameters where every prior step returned nothing:

```text
Resolving account_id for CheckLineStatus:
  1. SessionContext       → found (authenticated session)      ✓ done
Resolving postcode:
  1. SessionContext       → not present
  2. ConversationInfer    → "my BT1 connection" → "BT1"        ✓ done
Resolving device_id:
  1. SessionContext       → not present
  2. ConversationInfer    → nothing found
  3. BackendPrefill       → lookup by account_id → "RTR-0042"  ✓ done

→ zero questions asked for a 3-parameter action
```

This is where most bots feel dumb — they ask for information they could
derive. The resolution chain makes the derivation explicit and auditable.

**Proactive context sweep** — `BackendPrefill` is not limited to parameter
resolution. Before asking the user any breaking question, the engine can run
a predefined set of context-fetch actions to pre-populate session context:

```text
Session starts: "service is down"
  → proactive sweep:
      FetchLastLogLines(n=10)    → activates: error=timeout, service=auth
      RunHealthCheck()           → activates: database=unreachable
      FetchServiceStatus()       → activates: dependency=payment_gateway, status=degraded
  → graph activation with pre-fetched context
  → path narrows to: database_connectivity_issue  [confidence: 0.84]
  → no breaking questions needed — context was sufficient
```

The sweep is defined as a list of zero-argument context actions on the
knowledge graph. Each result maps to concept node activations via the same
adapter pattern as any other event. Breaking questions only fire if the
swept context still leaves the path ambiguous — the user is the last resort,
not the first.

**Failed action revert before learning** — when a session involves multiple
action attempts and none resolve the issue, the system enforces a clean state
before learning:

1. Every revertable action that did not lead to a confirmed resolution is
   rolled back before the session closes.
2. Only the final confirmed path — the one the user explicitly confirmed as
   working — is positively reinforced.
3. All intermediate failed paths receive negative reinforcement independently.
4. If no path was ever confirmed (full escalation), no positive reinforcement
   is applied. Only weak memory entries and negative signals are recorded.

This ensures the graph never learns a cumulative "do everything" path. The
learned path is always: initial context → working action → confirmed outcome.
Intermediate failed attempts are noise, not signal.

```text
Session: 3 attempts, attempt 3 confirmed

  attempt 1: scale_pool          → rejected → reverted → weight decays
  attempt 2: add_index           → rejected → reverted → weight decays
  attempt 3: kill_transaction    → confirmed → weight increases

  Graph learns: timeout → lock_contention → kill_transaction
  Graph does NOT learn: timeout → scale_pool → add_index → kill_transaction
```

**Escalation payload** — when a path terminates at an `Escalation` node,
the system assembles a structured handoff context rather than a bare message.
This eliminates the most common support frustration: being asked to repeat
information already given.

```rust
struct EscalationPayload {
    // summary is rendered at handoff time from node context — not stored as user-originated text
    detected_goal_node: u32,              // node ID of the inferred goal concept
    attempted_paths:    Vec<String>,      // path labels tried and rejected
    confirmed_facts:    Vec<(String, String)>, // key-value pairs confirmed during session
    missing_info:       Vec<String>,      // parameters never resolved
    confidence:         f32,              // engine confidence at time of escalation
    session_id:         String,           // link to full session record
}
```

`detected_goal_node` is a node ID from the knowledge graph — a curated concept label
like `diagnose_connectivity_loss`, not a free-text description of what the user said.
The human-readable `summary` line shown at handoff is rendered by the adapter from
the node label and confirmed facts; it is never stored in the payload itself.

Example escalation payload handed to a human agent:

```text
detected_goal:   diagnose_connectivity_loss  [node 47]
attempted_paths: ["outage_detected (ruled out)", "line_fault (inconclusive)"]
confirmed_facts: [("postcode", "BT1 4AB"), ("device_id", "RTR-0042"),
                  ("scope", "all_devices"), ("duration", "absent")]
missing_info:    ["last_router_reboot_time"]
confidence:      0.38
session_id:      "2026-04-01-007"
```

The human agent sees context immediately; no repeated questions.

Actions are defined externally to the graph — the graph contains only the
contract; the executor holds the implementation. This makes every possible
system action enumerable and auditable before deployment.

### 3.5 Response Envelope

Every output from the engine — whether an answer, a breaking question, an
action confirmation, or an escalation — is wrapped in a structured
`ResponseEnvelope`. A plain text stream is insufficient for any interface
beyond a raw CLI.

```rust
struct ResponseEnvelope {
    message:        String,             // primary text shown to the user
    confidence:     ConfidenceLevel,    // explicit certainty state (see §4.5)
    state:          SessionState,       // current session lifecycle stage
    ui:             Vec<UIComponent>,   // optional interface elements
    actions:        Vec<ActionOption>,  // available next steps the user can take
    requires_input: bool,               // whether the engine is waiting on the user
    trace:          Option<ReasoningTrace>, // populated when --explain is active
}

enum SessionState {
    Active,          // query in progress
    AwaitingInput,   // engine asked a breaking question or needs a parameter
    ActionPending,   // action contract selected, execution layer not yet called
    Resolved,        // goal confirmed
    Escalated,       // handed off with full context
    Abandoned,       // timed out or user exited without resolution
}

struct UIComponent {
    kind:    UIKind,    // Button | Toggle | Form | StatusCard | Diagram
    label:   String,
    payload: String,   // action token or branch label to submit on selection
}

struct ActionOption {
    label:    String,  // e.g. "Reboot router"
    contract: String,  // action name from the action contract system (§3.4)
    urgency:  u8,      // 0–10; higher options are rendered more prominently
}
```

The `ResponseEnvelope` is what adapters (CLI, HTTP, voice, web chat) consume.
Each adapter renders it differently:

| Adapter   | `message`      | `ui`                          | `actions`               |
| --------- | -------------- | ----------------------------- | ----------------------- |
| CLI       | printed text   | ignored                       | printed as `[1/2/n]`    |
| Web chat  | chat bubble    | rendered as buttons/forms     | rendered as action cards|
| Voice     | spoken via TTS | ignored                       | read as numbered choices|
| HTTP API  | JSON field     | JSON array                    | JSON array              |

This single struct is the contract between the engine and every surface it
runs on. Adding a new interface type requires only a new adapter — the engine
is unchanged.

### 3.6 Policy Engine

The reasoning engine is powerful enough to select destructive or irreversible
actions. A `PolicyEngine` sits between the reasoning layer and the execution
layer and enforces hard constraints before any action runs.

```rust
struct PolicyEngine {
    rules: Vec<PolicyRule>,
}

struct PolicyRule {
    action_pattern: String,         // glob match against action name, e.g. "Cancel*"
    required_permission: Permission,// None | Verified | Authenticated | Admin
    rate_limit: Option<RateLimit>,  // max invocations per window
    requires_confirmation: bool,    // force explicit user confirmation even if confident
    rollback_available: bool,       // whether the action can be undone
}

enum Permission { None, Verified, Authenticated, Admin }

struct RateLimit { max: u32, window_seconds: u32 }
```

Example policy rules:

```text
CancelService      → Permission: Authenticated, confirmation: true, rollback: false
RebootRouter       → Permission: Verified,      confirmation: false, rollback: true
CheckLineStatus    → Permission: None,           confirmation: false, rollback: true
ScheduleEngineer   → Permission: Verified,       confirmation: true,  rollback: true
```

When the engine selects an action contract, the policy engine evaluates it
before the execution layer is called. A blocked action does not fail silently —
it routes back to the graph as a `permission_denied` activation, which may
trigger a re-authentication breaking question or an escalation node.

Policy rules are stored in `policies.json` alongside the knowledge files and
are evaluated at runtime, not at graph-build time. This means a security rule
can be updated without rebuilding the graph.

---

## 3.7 Deployment Configuration Matrix

Every Engram deployment independently locks or opens four axes:

- **Context nodes** — whether new concept/solution nodes can be added at runtime
- **Actions** — whether new action contracts can be registered at runtime
- **Graph** — whether edge weights and connections learn from live session outcomes
- **Input mode** — whether breaking question responses accept only listed branch choices (`Constrained`) or any free-form input processed through the tokeniser (`Open`)

The first three axes govern what the graph can become. The fourth governs how users interact with it during a session.

**Input mode detail:**

| Mode | Breaking question accepts | Best for |
| --- | --- | --- |
| `Constrained` | Only listed branch choices (yes/no, numbered options) | Voice assistants, compliance routing, call-centre co-pilots — predictable paths, no ambiguous input |
| `Open` | Any input — free text, pasted log lines, error messages, file content — tokenised and activated as additional context | Developer tools, technical diagnostics, LLM mesh — richer context yields better path narrowing |

In `Open` mode, a user who pastes a stack trace instead of answering "is it intermittent?" is not breaking the flow — the tokeniser processes the paste and updates the activation vector, potentially resolving the ambiguity without a formal answer at all.

The four axes combine to 16 deployment configurations. The three graph axes from the original matrix remain valid independently of input mode — the table below shows representative combinations rather than all 16:

| Context | Actions | Graph | Input | Character | Natural use case |
| ------- | ------- | ----- | ----- | --------- | ---------------- |
| Locked | Locked | Locked | Constrained | Pure inference, fully auditable, controlled UX | Compliance routing, regulated environments, voice |
| Locked | Locked | Locked | Open | Pure inference, auditable, developer-friendly | CLI tools, technical diagnostics |
| Locked | Locked | Learning | Constrained | Stable domain, self-optimising, predictable UX | Industrial agent, support bot, IVR |
| Locked | Locked | Learning | Open | Stable domain, self-optimising, rich input | On-call tooling, developer assistants |
| Open | Locked | Learning | Open | LLM extends vocabulary, paths self-optimise | LLM-assisted knowledge distillation |
| Open | Open | Learning | Open | Fully adaptive — LLM teaches graph at every layer | Shareable LLM memory artifact |

**Locking mechanics:** each axis has a corresponding flag in the deployment
configuration. Locked axes reject write operations at the API boundary — no
silent mutation. Open axes accept writes but route new nodes and actions through
the provisional state (see §11 Weak Memory) until confirmed by sufficient sessions.

**Provisional nodes from LLM authoring:** when context or actions are open, an LLM
can propose new nodes via `engram.add_node()` / `engram.add_action()`. These enter
as `NodeKind::Latent` with zero confirmed sessions — visible in `--explain` and
`engram latent`, but below any confidence threshold until independently validated.
They earn weight through subsequent session confirmations, the same as any other path.

**The shareable artifact property:** the fully-open configuration (bottom row) means
an LLM reasoning with Engram over many sessions produces a portable, versioned,
human-inspectable knowledge graph encoding what it learned. That graph can be
exported, shared, merged with another instance's graph, audited, or rolled back —
a durable memory artifact, not a context window that resets.

---

## 4. Question Processing Pipeline

Input:

```text
Why does Rust complain about borrowing?
```

### 4.1 Tokenization and Normalization

Tokenization maps input text to node IDs. This is the primary NLP challenge
for human-originated freetext input — synonym handling, paraphrase, typos,
and domain drift all affect recall. There is a quality spectrum of approaches,
each independent of the graph engine:

| Approach | Mechanism | Recall | Size penalty | Phase |
| --- | --- | --- | --- | --- |
| Alias map | Hand-curated regex/lookup table | Low — misses paraphrase | None | Current |
| BM25 + n-grams | TF-IDF scored token overlap, bigram/trigram matching | Medium | None | Phase 13 |
| Static embeddings | word2vec / GloVe nearest-neighbour | High | ~10–30 MB | §20.1 |
| Sentence embeddings | all-MiniLM-L6-v2 class, ~22 MB | Very high | ~22 MB | §20.1 |

The graph engine is unchanged regardless of which tokenizer tier is in use —
only the quality of the activation seed vector changes. A deployment that
prioritises the 100 MB total budget can hold a trimmed embedding vocabulary
for its specific domain at well under 30 MB.

**For MCP and agent callers, tokenization is not the bottleneck** — see §2.6.
The LLM has already performed semantic extraction; the `nodes` parameter
arrives as a typed array of concept identifiers that map directly to node IDs.

For human freetext, stop words are removed and terms are stemmed or matched by alias:

```text
rust  →  rust
complain  →  (discarded)
borrowing  →  borrow
```

### 4.2 Context Activation

Matched tokens activate corresponding nodes with initial scores:

```text
borrow_checker:  0.90
compile_error:   0.60
rust:            0.45
```

### 4.3 Activation Propagation

Activation spreads forward through outgoing edges, attenuated by edge weight
and a decay factor $\lambda$:

$$a_{\text{target}} = a_{\text{source}} \times w_{\text{edge}} \times \lambda$$

Where $\lambda \in (0, 1)$ prevents runaway propagation (default: $\lambda = 0.85$).

State after one propagation step:

```text
mutable_reference_conflict:  0.90 × 0.81 × 0.85 = 0.620
lifetime_mismatch:           0.90 × 0.63 × 0.85 = 0.482
```

### 4.3.1 Activation Propagation as a Degenerate Attention Mechanism

The propagation formula is structurally equivalent to a single-head sparse attention step:

```text
Transformer attention (dense, all-pairs):
  score(q, k) = QKᵀ / √d  →  softmax  →  weighted sum of values

Engram activation propagation (sparse, graph-constrained):
  a_target = a_source × w × λ  →  ranking  →  top candidate nodes
```

Both operations: take a query signal, route it through a learned weight matrix, produce
a weighted selection over candidates. The differences are structural, not fundamental:

| Property | Transformer attention | Engram propagation |
| --- | --- | --- |
| Weight matrix | Dense (all-pairs) | Sparse (edges only) |
| Weight learning | Backpropagation | Session reinforcement |
| Output | Probability distribution | Ranked discrete candidates |
| Determinism | Stochastic (softmax + sampling) | Deterministic |
| Interpretability | Opaque | Every weight is a named, inspectable edge |

The edge weight `w` *is* the attention weight. It was designed from a knowledge graph
perspective, but the mathematical role is identical.

**Implication for specialist graphs:** Each specialist graph trained on different session
data develops different edge weight distributions — the auth graph becomes highly sensitive
to auth tokens, the billing graph to billing tokens. Different specialists implement
different attention patterns over the same input vocabulary. This emerges automatically
from training data, without any explicit sensitivity configuration.

This is a computationally reasonable model of domain expertise: a specialist's knowledge
is encoded as a particular sparse attention pattern over concepts, shaped by what they
have confirmed as correct in their domain.

### 4.4 Candidate Solution Ranking

Leaf `Solution` nodes are ranked by accumulated activation:

```text
solution: mutable_reference_conflict  →  score: 0.82
solution: lifetime_mismatch           →  score: 0.63
```

If the top score exceeds the **answer threshold** $\theta_a$ (default 0.75),
the answer is returned directly.  
Otherwise the system enters the **Breaking Question** phase.

### 4.5 Confidence State Machine

The thresholds $\theta_a$ and $\theta_d$ already exist in the system, but
they currently drive implicit branching. To be production-grade, confidence
must be an **explicit named state** that determines observable system
behaviour — not just a scalar compared against a constant.

```rust
enum ConfidenceLevel {
    High,    // top score ≥ θ_a  AND  gap to second ≥ θ_d
    Medium,  // top score ≥ θ_a  BUT  gap to second < θ_d
    Low,     // top score < θ_a
    Unknown, // no candidates reached activation at all
}
```

Behaviour is fully determined by state, not by scattered threshold checks:

| State     | Trigger                          | Engine behaviour                           |
| --------- | -------------------------------- | ------------------------------------------ |
| `High`    | top score high, gap wide         | Return answer immediately                  |
| `Medium`  | top score high, gap narrow       | Present composite answer, ask 1 or 2       |
| `Low`     | top score below answer threshold | Enter breaking question flow               |
| `Unknown` | no candidates surfaced           | Noise handling / context expansion (SS15.5)|

The `ConfidenceLevel` is included in the `ResponseEnvelope` (§3.5) so every
adapter can render uncertainty visibly:

```text
[CLI]
  ? Are multiple parts of the code mutably borrowing the same value?
    Confidence: LOW — two candidates within 0.12 of each other

[Web]
  Confidence badge: 🟡 Medium — tap to see both possibilities

[Voice]
  "I have two possible answers. Which sounds more like your situation? ..."
```

Confidence is never hidden from the user. A system that knows it is uncertain
and says so is more trustworthy than one that projects false certainty.

---

## 5. Engram as a Confidence-Weighted Deterministic Finite State Machine

Engram is not merely *inspired* by automata theory — it is structurally an FSM
with two properties that extend the classical model.

### 5.1 The Classical FSM Correspondence

A classical deterministic FSM is defined as a 5-tuple:

```text
M = (Q, Σ, δ, q₀, F)
```

Where:

- `Q` — finite set of states
- `Σ` — input alphabet
- `δ: Q × Σ → Q` — transition function
- `q₀` — initial state
- `F ⊆ Q` — set of accepting states

Engram maps to this exactly:

| FSM component | Engram equivalent |
| --- | --- |
| `Q` — states | Graph nodes (`Concept`, `Question`, `Solution`, `Escalation`) |
| `Σ` — alphabet | Tokenised input (node ID activation vectors) |
| `δ` — transition function | Weighted edge traversal, resolved by activation ranking |
| `q₀` — initial state | Context activation from input tokens |
| `F` — accepting states | `Solution` and `Escalation` leaf nodes |

The `SessionState` enum (`Active → AwaitingInput → ActionPending → Resolved / Escalated / Abandoned`)
is the session-level FSM layered on top of the graph-level FSM. Both are deterministic:
given the same input and the same graph, the same path is always taken.

### 5.2 The Two Extensions

**Extension 1: Confidence-weighted transitions.**
Classical FSM transitions are binary — a transition either exists or does not. Engram
edges carry a `weight` and `confidence` score. When multiple transitions are valid from
the current state, the engine ranks them and selects the highest. This is still
deterministic (same ranking, same graph, same result) but adds a principled way to
handle ambiguity that classical FSMs cannot express without explicit branching for every
possible input.

```text
Classical FSM:   state A --input x--> state B   (binary — exists or not)
Engram:          state A --input x--> state B    (weight: 0.81, confidence: 0.75)
                           --input x--> state C  (weight: 0.43, confidence: 0.51)
                 → selects B deterministically by ranking
```

**Extension 2: A self-organising transition table.**
In a classical FSM, `δ` is fixed at design time. In Engram with the Graph axis open,
`δ` updates from confirmed session outcomes via the Rescorla-Wagner-inspired learning
rule. The transition table self-organises — paths that resolve correctly strengthen,
paths that fail decay — without changing the determinism guarantee. At any point in
time, given the current weights, the system behaves deterministically.

```text
Classical FSM:   δ is fixed — designed once, never changes
Engram (locked): δ is fixed — frozen graph, pure inference, fully auditable
Engram (open):   δ evolves — edge weights update from sessions, structure is preserved
```

### 5.3 The Deployment Axis as FSM Configuration

The four deployment axes map precisely onto FSM properties:

| Axis | Locked | Open |
| --- | --- | --- |
| Context nodes | Q is fixed — no new states | Q can grow — new nodes enter as provisional states |
| Actions | F is fixed — no new accepting states | New `Solution` nodes can be added at runtime |
| Graph | δ is fixed — static transition table | δ evolves — edge weights update from sessions |
| Input mode | Σ is constrained — listed branch choices only | Σ is open — any input tokenised as context |

A fully locked deployment is a classical FSM with a learned transition table.
A fully open deployment is an extensible FSM that grows its state space, transition table,
and accepting states at runtime — all through the same provisional validation mechanism.

### 5.4 Implications

**Formal correctness.** Because Engram is an FSM, it inherits 60 years of formal
automata theory. Properties like reachability ("can state B ever be reached from state A?"),
safety ("can an action contract C ever fire from state S?"), and completeness ("are all
inputs handled?") can be verified statically on the knowledge graph before deployment.
This is not possible with LLM-based systems.

**Human-machine interface.** Any interface that reduces to: receive input → transition
state → emit output → wait — is an FSM. Voice interfaces, IVR systems, embedded device
controllers, CLI tools, and web form flows all fit this model exactly. Engram provides
a single runtime for all of them, with the same graph driving every surface via the
`ResponseEnvelope` adapter pattern (§3.5).

**Generic deterministic automation.** The FSM framing makes the scope explicit: Engram
is applicable to any domain where the problem space is finite (or finitely extensible),
transitions are deterministic, and outcomes are verifiable. This is a broader class than
"AI assistant" — it includes industrial controllers, compliance routing, business process
engines, and protocol state machines.

---

## 6. Storage Backend Abstraction

The current implementation reads graph data from JSON files on disk. This is the right
default — simple, portable, version-controllable, human-readable — but it couples the
reasoning engine to a specific storage medium in a way that limits platform flexibility.

### 6.1 The Interface Boundary

The reasoning engine should not know or care where nodes, edges, and paths come from.
The correct design is a thin storage trait (interface) that the engine calls, with
concrete implementations behind it:

```text
ReasoningEngine
      │
      ▼
  GraphStore (trait)
      │
      ├── FileStore       — JSON files on disk (current)
      ├── MemoryStore     — in-process, no persistence (testing, embedded)
      ├── RestApiStore    — remote Engram instance or graph service over HTTP
      ├── DatabaseStore   — SQL / key-value / document store
      └── SocketStore     — Unix socket or named pipe (low-latency IPC)
```

The engine calls `graph.get_node(id)`, `graph.get_edges(src)`,
`graph.persist_session(outcome)` — the backing store is resolved at startup from
the deployment configuration. Switching from file to database requires no engine change.

### 6.2 What This Enables

**Platform deployments.** A `DatabaseStore` backed by PostgreSQL or SQLite makes the
graph queryable by external tools, auditable via standard SQL, and naturally backed up
by existing database infrastructure. A `RestApiStore` makes it possible to separate the
reasoning process from the graph data entirely — useful in multi-tenant or SaaS
deployments where each tenant owns their graph.

**Live graph updates.** A `RestApiStore` or `SocketStore` can stream graph updates from
a central authority to running instances without restart. The instance sees weight changes
the moment they are committed centrally.

**Hybrid stores.** A read-through cache over a remote store: hot nodes in memory, cold
nodes fetched on demand, writes propagated asynchronously. This is natural once the store
is an interface.

**Testing and simulation.** A `MemoryStore` pre-loaded with a known graph state allows
deterministic unit testing of reasoning paths without file system dependency.

### 6.3 Roadmap Note

This is not a Phase 1–8 concern — the file backend is sufficient through the initial
roadmap. The storage trait should be introduced when the first non-file deployment
requirement arises (likely a `DatabaseStore` for multi-user or SaaS use). Introducing
the trait too early would be premature abstraction; introducing it too late would require
invasive refactoring. The right trigger is the first concrete deployment that cannot be
satisfied by JSON files.

---

## 7. Multi-Instance Federation and Graph Merging

A single Engram instance handles a bounded domain. Real deployments often span multiple
domains, multiple teams, or multiple geographic locations — each with their own instance.
This section covers how instances can be connected and how their graphs can be combined.

### 7.1 Inter-Instance Communication

Instances communicate through action contracts, the same mechanism used for any other
external call. An action defined as `FetchFromEngram` with a `RestApiStore` endpoint
is structurally identical to any other action — it is enumerable, policy-gated, and
auditable. No special federation protocol is needed at the engine level.

```text
Instance A (auth domain)
  │
  └── action: FetchFromEngram(instance=billing, query="account_status")
        │
        ▼
      Instance B (billing domain)
        → returns: path, confidence, ruled-out candidates
        → result activates nodes in Instance A's session context
```

Communication transports map to the storage abstraction (§6):

| Transport | Use case |
| --- | --- |
| REST API | Cross-network, cross-team, multi-tenant |
| Unix socket / named pipe | Low-latency IPC on the same host |
| Shared database | Shared read, isolated write — useful for analytics aggregation |
| Graph delta stream | Asynchronous weight propagation (§7 in use_cases.md) |

The policy engine gates every inter-instance call identically to any other action:
permission level, rate limit, and confirmation requirement apply. An instance cannot
be interrogated by another instance unless the action is in the contract.

### 7.2 Graph Merging

When two instances have accumulated knowledge in overlapping domains, their graphs can
be merged into a single graph. This is the "melting" operation.

**Non-overlapping knowledge** is trivially additive: nodes and edges from both graphs
are combined, with no conflict. Path labels, node IDs, and action contracts are
unioned.

**Overlapping paths — weight resolution.** When both graphs contain an edge between
the same two concepts, the merged edge weight is computed from both sources:

```text
Graph A:  timeout → connection_pool   weight: 0.76, n=28
Graph B:  timeout → connection_pool   weight: 0.61, n=14

Merged:   weight = (0.76×28 + 0.61×14) / (28+14) = 0.71
          n     = 28 + 14 = 42
```

This is a session-count-weighted average — graphs with more confirmed sessions
contribute proportionally more to the merged weight. The result encodes the collective
experience of both instances without either dominating arbitrarily.

**Boundary deduplication.** Before merging, node and action identities must be aligned:

1. **Canonical node ID mapping** — nodes representing the same concept may have
   different IDs in each graph. A merge requires a mapping step: either by matching
   labels, by a shared ontology, or by manual curation for ambiguous cases.
2. **Action contract deduplication** — actions with the same name but different
   parameter schemas must be reconciled before merging. Divergent schemas are a merge
   conflict that requires explicit resolution, not silent overwriting.
3. **Provisional nodes** — nodes that are provisional (low confirmed sessions) in
   either graph retain their provisional status in the merged graph. They do not
   inherit the other graph's confidence.

**Merge is not always correct.** Two graphs trained on different populations may have
legitimately different weights for the same path — one team's experience genuinely
differs from another's. A merge conflates this. The right question before merging is:
*should these two populations share a transition table?* If the answer is no, federation
(§7.1) is more appropriate than merging — instances remain separate and query each other
when needed.

### 7.3 Roadmap Note

Inter-instance communication via action contracts (§7.1) requires no new engine
primitives — it is a knowledge graph authoring task once the `RestApiStore` backend
exists (§6). Graph merging (§7.2) is a CLI operation (`engram merge graph-a.json graph-b.json`)
that can be specified and tested independently of the runtime. Both are natural follow-ons
to the §8 roadmap in use_cases.md, not prerequisites for it.
