# Engram — Disambiguation and Goal Tracking

*Part of the [Engram design documentation](proposal.md).*

---

## 5. Breaking Questions

Breaking questions are **systematic decomposition questions** that partition
the active solution space into mutually exclusive labeled branches. They differ
from simple yes/no clarification: each possible answer commits the session to
a **named context path** with its own tag set.

### 5.1 Purpose

When two or more candidate solutions have activation scores within a
**disambiguation threshold** $\theta_d$ (default 0.15) of each other,
the system cannot choose reliably. A breaking question eliminates the
ambiguity by asking the user to select a branch dimension.

### 5.2 Breaking Question Structure

```rust
struct BreakingQuestion {
    id:            u32,
    label:         String,           // e.g. "ownership_dimension"
    prompt:        String,           // text shown to the user
    branches:      Vec<Branch>,
}

struct Branch {
    answer_token:  String,           // expected user token ("yes", "no", "multiple")
    target_node:   u32,              // graph node to activate on this answer
    path_label:    String,           // label committed to session path
    tags:          Vec<String>,
}
```

### 5.3 Decomposition Strategy

The system selects the breaking question that **maximally separates** the
candidate solutions. Selection criterion: choose the question whose branches
assign the highest-scoring candidate to one branch and all others to separate
branches.

```text
Candidate solutions:
  A: mutable_reference_conflict  score: 0.82
  B: lifetime_mismatch           score: 0.70  (delta = 0.12 < θ_d)

Breaking question selected:
  label:  "ownership_dimension"
  prompt: "Are multiple parts of the code holding a mutable reference
           to the same value simultaneously?"

  Branch yes  →  mutable_reference_conflict  path_label: "ownership_violation"
  Branch no   →  lifetime_mismatch           path_label: "lifetime_scope"
```

### 5.4 Question Labeling

Every breaking question node carries a **domain label** that identifies the
conceptual dimension being questioned:

```text
ownership_dimension     — mutation / aliasing questions
lifetime_dimension      — scope and borrow duration questions
concurrency_dimension   — async, threads, locks
type_dimension          — trait bounds, generics, type mismatch
```

Labels are stored on the `Question` node and propagated to the session's
active path. This enables future sessions with the same domain signature to
**skip already-answered dimensions**.

---

## 6. Context Path Labeling and Tagging

A **context path** is the sequence of nodes traversed from the entry
concept to the selected solution. Path labeling assigns persistent semantic
identifiers to these routes so they can be named, reused, and compared.

### 6.1 Path Definition

```rust
struct ContextPath {
    id:          u32,
    name:        String,              // human-readable, e.g. "rust_ownership_violation"
    node_ids:    Vec<u32>,            // ordered node sequence
    tags:        Vec<String>,         // semantic tag set
    usage_count: u32,
    avg_confidence: f32,
}
```

Example:

```text
ContextPath {
  name:     "rust_ownership_violation"
  nodes:    [rust → borrow_checker → mutable_reference_conflict → solution_001]
  tags:     ["ownership", "mutation", "single_threaded", "rust"]
  usage:    22
}
```

### 6.2 Tag Taxonomy

Tags are organized in three tiers:

| Tier    | Examples                                          | Purpose                        |
| ------- | ------------------------------------------------- | ------------------------------ |
| Domain  | `rust`, `python`, `sql`                           | Language or technology area    |
| Pattern | `ownership_violation`, `deadlock`, `type_error`   | Structural problem class       |
| Scope   | `single_threaded`, `async`, `distributed`         | Execution context              |

### 6.3 Path Matching for Fast Resolution

When a new query activates a set of nodes, the system checks whether those
nodes are a **subset of a known labeled path**. If a match is found with
sufficient tag overlap, the known path is proposed directly:

```text
Active nodes:  {rust, borrow_checker}
Known path match: "rust_ownership_violation"  (overlap: 0.88)
→ Propose solution from known path, skip full propagation
```

This acts as a **path-level cache** and speeds up repeated queries.

### 6.4 Tag Propagation to Sessions

Each session records the labels of the paths it traversed:

```json
{
  "session_id": "2026-03-06-001",
  "path_labels": ["ownership_violation", "single_threaded"],
  "breaking_questions_asked": ["ownership_dimension"],
  "outcome": "confirmed"
}
```

Session tag histories are used to:

- detect user-specific knowledge gaps (which dimensions are asked repeatedly)
- bias future breaking question selection for that user profile
- feed the **Latent Node Discovery** algorithm with co-occurrence signals

### 6.5 Path Label Evolution

When the reinforcement system updates an edge's weight, the labels on all
paths containing that edge are marked **stale** if confidence drops below
$\theta_c = 0.4$. Stale paths are re-evaluated against current edge weights
rather than served from cache.

---

## 7. Clarification Mechanism

The clarification mechanism is a simplified form of breaking questions for
cases where the activation gap is large enough to identify a single dominant
candidate, but the confidence is below the answer threshold.

Rather than a full branch decomposition, a single targeted yes/no question
is asked against the top candidate:

```text
Bot: Is the error about multiple mutable references? [ownership_dimension]
User: yes
→ Path confirmed: "rust_ownership_violation"
→ Edge weights reinforced along confirmed path
```

The `[ownership_dimension]` label is shown in explanation mode to make
the reasoning dimension visible to the user.

---

## 7.5 Goal Tracking

A **goal** spans multiple exchanges and may contain parallel sub-queries. It
differs from a single question/answer session in that it has a lifecycle and
can be revised or extended mid-conversation without restarting graph traversal.

### 7.5.1 Goal Structure

```rust
struct Goal {
    id:                 u32,
    description:        String,        // e.g. "diagnose unexpected £65 charge"
    status:             GoalStatus,    // Open | Resolved | Revised | Escalated
    sub_sessions:       Vec<SessionId>,// ordered sessions under this goal
    active_path_labels: Vec<String>,   // accumulated context across sub-sessions
    created_at:         Timestamp,
    revised_at:         Option<Timestamp>,
}

enum GoalStatus { Open, Resolved, Revised, Escalated }
```

### 7.5.2 Goal Revision

Mid-conversation, the user may reframe the problem. The system does not restart
graph traversal — it re-enters propagation with existing context already biased
by previously confirmed breaking question answers:

```text
Turn 1: "my internet is slow"         → goal: diagnose_connectivity
Turn 3: "actually it drops entirely"  → goal revised: connectivity_loss
  → breaking questions already answered are retained in activation context
  → system routes toward connectivity_loss nodes from current activation state
  → no repeated questions for dimensions already resolved
```

### 7.5.3 Parallel Sub-Goals

When a query activates two unrelated high-confidence paths simultaneously,
the system may open two sub-goals rather than forcing a single branch:

```text
User: "billing is wrong and my router won't connect"
  Sub-goal A: billing_dispute      (activation: 0.84)
  Sub-goal B: router_connectivity  (activation: 0.81)

→ System handles B first (higher urgency score), then returns to A
→ Both sub-sessions are logged under the same parent goal ID
```

Goal urgency scoring uses a configurable tag: solution nodes tagged
`urgency:high` (e.g. connectivity loss, outage) are prioritised over
`urgency:normal` (e.g. billing queries) when parallel sub-goals compete.

Goals are persisted to `goals.json` alongside `sessions.json`.

### 7.5.4 Urgency and Impact Scoring

Not all goals are equal. When parallel sub-goals compete for handling order,
or when a new query interrupts an open goal, the system uses two node-level
tags to determine priority:

```text
urgency:high    — time-sensitive; user is blocked right now  (e.g. no internet)
urgency:normal  — informational; user can wait              (e.g. billing query)
impact:service  — affects core service delivery
impact:account  — affects account or billing only
```

Priority score (higher = handle first):

```text
priority = urgency_weight * urgency + impact_weight * impact
           (defaults: urgency_weight = 0.7, impact_weight = 0.3)
```

Example ordering:

```text
Sub-goal A: billing_dispute       urgency:normal, impact:account  → priority 0.30
Sub-goal B: connectivity_loss     urgency:high,   impact:service  → priority 1.00

→ B handled first; A queued with [deferred] marker in ResponseEnvelope
```

Urgency and impact tags are defined on `Solution` and `Escalation` nodes in
the knowledge files — not hardcoded in the engine. Changing the urgency of a
domain is a one-line edit to the knowledge file.
