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

| Kind           | Description                                              |
| -------------- | -------------------------------------------------------- |
| `Concept`      | A domain term or context anchor                          |
| `Question`     | A breaking or clarifying question node                   |
| `Solution`     | A leaf node with a text answer or typed action contract  |
| `Latent`       | Auto-discovered hidden concept                           |
| `Escalation`   | Path terminus that exports structured context for handoff|

### 3.3 Edge Structure

```rust
struct Edge {
    source:      u32,
    target:      u32,
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

Stop words are removed; terms are stemmed or matched by alias:

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
