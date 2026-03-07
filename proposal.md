# Proposal: Context Graph Self-Learning CLI Chatbot

**Status:** Draft v1.2  
**Goal:** A lightweight, deterministic, self-improving CLI chatbot with a context graph, breaking question decomposition, and path labeling.  
**Constraints:** <100 MB memory, fully explainable, incremental learning, no external model dependency.

---

## 1. Objective

Design a **lightweight conversational reasoning system** that:

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

```
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

## 3. Knowledge Representation

### 3.1 Context Graph

The knowledge base is a **weighted directed graph**. Nodes are concepts;
edges are contextual transitions.

```
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
    kind:         NodeKind,     // Concept | Question | Solution | Latent
    activation:   f32,          // transient score during query processing
    tags:         Vec<String>,  // semantic tags, e.g. ["ownership", "rust"]
}
```

Node kinds:

| Kind       | Description                                    |
| ---------- | ---------------------------------------------- |
| `Concept`  | A domain term or context anchor                |
| `Question` | A breaking or clarifying question node         |
| `Solution` | A leaf node with an actionable answer          |
| `Latent`   | Auto-discovered hidden concept                 |

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

```
borrow_checker → mutable_reference_conflict
  weight:      0.81
  confidence:  0.75
  usage_count: 14
  path_labels: ["ownership_violation", "single_threaded"]
```

---

## 4. Question Processing Pipeline

Input:

```
Why does Rust complain about borrowing?
```

### 4.1 Tokenization and Normalization

Stop words are removed; terms are stemmed or matched by alias:

```
rust  →  rust
complain  →  (discarded)
borrowing  →  borrow
```

### 4.2 Context Activation

Matched tokens activate corresponding nodes with initial scores:

```
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

```
mutable_reference_conflict:  0.90 × 0.81 × 0.85 = 0.620
lifetime_mismatch:           0.90 × 0.63 × 0.85 = 0.482
```

### 4.4 Candidate Solution Ranking

Leaf `Solution` nodes are ranked by accumulated activation:

```
solution: mutable_reference_conflict  →  score: 0.82
solution: lifetime_mismatch           →  score: 0.63
```

If the top score exceeds the **answer threshold** $\theta_a$ (default 0.75),
the answer is returned directly.  
Otherwise the system enters the **Breaking Question** phase.

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

```
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

```
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

```
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

```
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

```
Bot: Is the error about multiple mutable references? [ownership_dimension]
User: yes
→ Path confirmed: "rust_ownership_violation"
→ Edge weights reinforced along confirmed path
```

The `[ownership_dimension]` label is shown in explanation mode to make
the reasoning dimension visible to the user.

---

## 8. Learning from Interaction

Every completed session updates the graph along the confirmed path.

Example:

```
Q: Rust borrow error
Bot: [breaking] Are multiple parts of the code mutably borrowing the same value?
User: yes
Bot: Only one mutable reference is permitted at a time in the same scope.
User: confirmed — that solved it
```

Graph updates applied:

```
borrow_checker → mutable_reference_conflict
  weight      += Δw
  confidence  += Δc
  usage_count += 1
path "rust_ownership_violation"
  usage_count += 1
  avg_confidence recalculated
```

---

## 9. Reinforcement Strategy

### 9.1 Positive Reinforcement (confirmed solution)

$$w' = w + \alpha \cdot (1 - w)$$

$$c' = c + \beta \cdot (1 - c)$$

Where $\alpha$ is the learning rate (default 0.05) and $\beta$ is the
confidence step (default 0.03). The $(1 - w)$ factor prevents saturation
near 1.0.

### 9.2 Negative Reinforcement (rejected solution)

$$w' = w - \alpha \cdot w$$

$$c' = c - \beta \cdot c$$

The symmetric formula ensures weights decay proportionally, preventing
collapse to zero.

### 9.3 Path-Level Reinforcement

When a named path is confirmed, all edges along it receive a reduced
reinforcement scaled by path length $n$:

$$\Delta w_{\text{path}} = \frac{\alpha}{n}$$

This prevents long paths from being over-reinforced compared to short paths.

---

## 10. Latent Node Discovery

Latent nodes represent **hidden shared concepts** that emerge from
repeated co-activation patterns across multiple distinct paths.

### 10.1 Co-occurrence Monitoring

For every pair of nodes $(A, B)$ that are activated in the same session,
increment a co-occurrence counter $\text{co}(A, B)$.

### 10.2 Similarity Score

$$\text{sim}(A, B) = \frac{\text{co}(A, B)}{\sqrt{\text{freq}(A) \cdot \text{freq}(B)}}$$

This is the Jaccard-normalized co-occurrence. Values approaching 1.0 indicate
that $A$ and $B$ almost always appear together.

### 10.3 Latent Node Creation

When a group of nodes $\{A, B, C, \ldots\}$ all share pairwise similarity
above a threshold $\theta_L = 0.65$:

1. Create a new `Latent` node $L$ with an auto-generated label
2. Add edges $A \to L$, $B \to L$, $C \to L$ with initial weight $0.5$
3. Tag $L$ with the intersection of the tag sets of $A$, $B$, $C$
4. Label the new node for human review (surfaced in explanation mode)

Example:

```
High co-occurrence group: {tokio_deadlock, database_deadlock, thread_deadlock}
Common tags: ["waiting", "lock"]
→ Create latent node: "deadlock"
→ Edges: tokio_runtime → deadlock
         database_runtime → deadlock
         thread_runtime → deadlock
→ Tag:   ["concurrency", "lock", "waiting"]
```

---

## 11. Weak Answer Memory

Incorrect or uncertain answers are stored rather than discarded.

### 11.1 Storage Format

```json
{
  "id": "wm-0042",
  "question": "Why does rust borrow fail?",
  "tokens": ["rust", "borrow"],
  "attempted_path": "rust_lifetime_scope",
  "attempted_solution": "lifetime issue",
  "status": "uncertain",
  "session_id": "2026-03-06-001",
  "correction": null
}
```

### 11.2 Promotion to Main Graph

When a user later provides the correct answer:

```
User: Actually the issue was mutable reference conflict
```

The system:

1. Locates the weak memory entry by session or question hash
2. Resolves the correct path ("rust\_ownership\_violation")
3. Applies positive reinforcement to the correct path
4. Applies negative reinforcement to the incorrect path
5. Updates the entry status to `"resolved"` and archives it

---

## 12. CLI Behavior

### 12.1 Interactive Loop

```
$ chattie

chattie> why rust borrow error?

[activation] borrow_checker: 0.90, compile_error: 0.60
[ambiguous]  mutable_reference_conflict: 0.82, lifetime_mismatch: 0.70

? Are multiple parts of the code mutably borrowing the same value?
  [ownership_dimension]

> yes

[path] rust_ownership_violation (confirmed)

Answer: Only one mutable reference is permitted at a time in the same scope.

Was this helpful? [y/n] y
[graph updated]
```

### 12.2 Single Query Mode

```
$ chattie "why rust borrow error?"

Possible cause:  multiple mutable references (score: 0.82)
Path:            rust_ownership_violation
Confidence:      0.75
```

### 12.3 Explanation Mode

```
$ chattie --explain "why rust borrow error?"

Reasoning path:
  rust  [domain: rust]
  → borrow_checker  [concept]
  → mutable_reference_conflict  [pattern: ownership_violation]

Path label:  rust_ownership_violation
Tags:        ownership, mutation, single_threaded, rust
Confidence:  0.75  |  Usage: 22

Solution:
  Only one mutable reference is permitted at a time in the same scope.
```

---

## 13. Memory Layout

```
knowledge/
  nodes.json          — node definitions with tags
  edges.json          — edges with weights, confidence, path_labels
  paths.json          — named context paths with tag sets
  questions.json      — breaking question nodes and branch definitions
  solutions.json      — solution text associated with leaf nodes
  weak_memory.json    — uncertain and incorrect answer records
  sessions.json       — session history with path labels traversed
```

### 13.1 nodes.json

```json
[
  { "id": 1, "label": "rust",       "kind": "Concept",  "tags": ["rust"] },
  { "id": 2, "label": "borrow_checker", "kind": "Concept", "tags": ["rust", "ownership"] },
  { "id": 3, "label": "mutable_reference_conflict", "kind": "Solution",
    "tags": ["ownership", "mutation"] }
]
```

### 13.2 edges.json

```json
[
  { "src": 2, "dst": 3, "weight": 0.81, "confidence": 0.75,
    "usage_count": 14, "path_labels": ["rust_ownership_violation"] }
]
```

### 13.3 paths.json

```json
[
  {
    "id": 1,
    "name": "rust_ownership_violation",
    "node_ids": [1, 2, 3],
    "tags": ["ownership", "mutation", "single_threaded", "rust"],
    "usage_count": 22,
    "avg_confidence": 0.76
  }
]
```

---

## 14. Initial Knowledge Base

Start with a curated seed of 100–300 nodes covering common problem patterns.

**Seed concepts:**

```
rust, borrow_checker, mutable_reference, lifetime, compile_error,
tokio, async, deadlock, mutex, thread, ownership, trait, generic,
type_error, null_pointer, index_out_of_bounds, stack_overflow
```

**Seed solutions:**

```
mutable_reference_conflict:  "Only one mutable reference is permitted at a time."
lifetime_issue:              "Check that all borrows are within the owner's scope."
deadlock:                    "Ensure lock acquisition order is consistent across threads."
```

**Seed breaking questions:**

```
ownership_dimension:
  prompt: "Are multiple parts of the code mutably borrowing the same value?"
  branches: yes → mutable_reference_conflict | no → lifetime_mismatch

concurrency_dimension:
  prompt: "Does the error occur only under concurrent execution?"
  branches: yes → deadlock | no → single_thread_error
```

**Seed paths:**

```
rust_ownership_violation:  [rust → borrow_checker → mutable_reference_conflict]
  tags: [ownership, mutation, single_threaded, rust]

rust_lifetime_scope:       [rust → borrow_checker → lifetime_mismatch]
  tags: [lifetime, scope, rust]
```

---

## 15. Automatic Context Expansion

When a query contains an unrecognized token, the system creates a provisional node:

```
Token:    "vectorization"
Action:   create Concept node "vectorization", tags: []
Connect:  programming → vectorization  (weight: 0.3, confidence: 0.1)
```

The node is flagged as **unconfirmed**. After three interactions that route
through it, confidence rises above the confirmation threshold and the node
is promoted to fully active. Tags and path memberships are assigned during
the promotion step.

---

## 16. Context Bias

Frequently reinforced paths become dominant. High-weight edges are traversed
first during propagation, effectively pre-selecting the most probable solution
before the full graph is evaluated.

```
borrow_checker → mutable_reference_conflict
  weight: 0.93   ← heavily reinforced
```

This is analogous to **learned bias in a neural network**: the system develops
a prior shaped by historical interactions.

To prevent over-bias locking out correct but rare paths, a small **exploration
noise** $\epsilon$ (default 0.02) is added to activation scores, ensuring
lower-weight edges occasionally participate in propagation.

---

## 17. Expected System Size

| Component          | Estimated Size |
| ------------------ | -------------- |
| nodes + edges      | 5–30 MB        |
| named paths        | <2 MB          |
| solutions          | 1–5 MB         |
| weak memory        | <5 MB          |
| session history    | <3 MB          |
| **Total**          | **< 45 MB**    |

Well below the 100 MB constraint, leaving room for graph growth.

---

## 18. Development Phases

Each phase produces a **fully working, inspectable system**. Nothing is
left in a half-built state at the end of a phase. Every phase either adds
new behavior or deepens existing behavior — it never breaks what the previous
phase established.

The phases are ordered so that the system is **useful from Phase 2 onward**
and progressively smarter from there.

---

### Phase 0 — Project Skeleton

**Goal:** A compilable binary with defined data structures and file I/O.
No reasoning logic yet.

Deliverables:

- `Node`, `Edge`, `ContextPath`, `BreakingQuestion`, `Branch` structs defined
- `knowledge/` directory layout established; loader reads JSON files on startup
- CLI parses arguments: interactive mode vs single-query mode vs `--explain`
- `chattie` binary runs, prints a greeting, and exits cleanly

Checkpoint:

```
$ chattie
chattie v0.1 — knowledge loaded: 0 nodes, 0 edges
chattie>
```

What this phase gives you: a buildable project with a clear structure you
can navigate before any logic exists.

---

### Phase 1 — Static Seed Knowledge Base

**Goal:** The system can answer questions using direct keyword lookup against
the seed knowledge base. No graph traversal yet.

Deliverables:

- Populate `knowledge/nodes.json`, `edges.json`, `solutions.json`,
  `questions.json` with the seed data from §14
- Tokenizer: split input, remove stop words, apply alias map
- Keyword matcher: find nodes whose label matches any input token
- If a matched node has kind `Solution`, return its text directly
- `--explain` prints the matched token and node label

Checkpoint:

```
$ chattie "mutable reference"
Answer: Only one mutable reference is permitted at a time in the same scope.
Path:   direct match → mutable_reference_conflict

$ chattie --explain "mutable reference"
Token match:  mutable_reference  →  Node #3 [Solution]
Answer:  Only one mutable reference is permitted at a time in the same scope.
```

What this phase gives you: a working (if naive) answering system you can
query against real seed data and inspect the raw knowledge files.

---

### Phase 2 — Graph Activation and Propagation

**Goal:** Answers are found by traversing the graph, not just by direct match.
The system can now reason one or more hops away from the input tokens.

Deliverables:

- Assign initial activation scores to matched concept nodes (§4.2)
- Propagate activation forward through outgoing edges using
  $a_{\text{target}} = a_{\text{source}} \times w \times \lambda$ (§4.3)
- Repeat propagation for a configurable depth (default: 4 hops)
- Rank all reached `Solution` nodes by accumulated activation (§4.4)
- Return the top-ranked solution if its score exceeds $\theta_a = 0.75$
- `--explain` prints the full activation trace with scores at each hop

Checkpoint:

```
$ chattie --explain "why rust borrow error"

Activation trace:
  rust          0.45  [concept]
  borrow_checker  0.90  [concept]
  → mutable_reference_conflict  0.90 × 0.81 × 0.85 = 0.620  [solution]
  → lifetime_mismatch           0.90 × 0.63 × 0.85 = 0.482  [solution]

Top solution (score 0.62 < θ_a 0.75): threshold not met — entering clarification
```

What this phase gives you: a real graph traversal engine. You can now watch
activation flow through the knowledge base and understand exactly why a
particular solution ranks highest.

---

### Phase 3 — Clarification Questions (Single-Branch)

**Goal:** When the top solution score is below $\theta_a$, the system asks
a single yes/no question to confirm or reject the top candidate. This is the
simplest form of disambiguation.

Deliverables:

- Load `questions.json`; associate each `Question` node with its `Branch` list
- Locate the clarification question linked to the current top candidate node
- Ask it; accept `yes` / `no` (and aliases: `y`, `n`, `true`, `false`)
- On `yes`: confirm the path, return the solution
- On `no`: drop the top candidate, re-rank remaining solutions, repeat
- `--explain` labels the question with its domain dimension tag

Checkpoint:

```
chattie> why rust borrow error

[score below threshold]
? Is the error about multiple mutable references?  [ownership_dimension]
> yes

Answer: Only one mutable reference is permitted at a time in the same scope.
```

What this phase gives you: an interactive system that handles ambiguity.
You can trace which question was chosen and why, and verify the branch logic
by answering `no` and watching the fallback candidate surface.

---

### Phase 4 — Breaking Questions (Multi-Branch Decomposition)

**Goal:** Replace single yes/no clarification with proper breaking questions
that partition the candidate set into labeled mutually exclusive branches (§5).

Deliverables:

- `BreakingQuestion` selection: choose the question that maximally separates
  candidates within $\theta_d = 0.15$ of each other (§5.3)
- Support branches beyond yes/no (e.g. `"multiple"`, `"async"`, `"never"`)
- Each branch commits to a `path_label` stored in the session's active context
- Chained questions: if the chosen branch still has ambiguity, ask again
- Maximum question depth: configurable (default: 3 questions per session)
- `--explain` shows all evaluated candidates and which branch eliminated them

Checkpoint:

```
chattie> deadlock in tokio

[ambiguous: tokio_deadlock 0.71, database_deadlock 0.69]
? Does the deadlock occur inside an async runtime?  [concurrency_dimension]
> yes

[branch: async_context  →  tokio_deadlock confirmed]

Answer: Avoid holding a mutex guard across an .await point.
Path:   tokio_runtime → deadlock → solution_deadlock_async
```

What this phase gives you: the full breaking-question engine. You can
inspect `questions.json` to see all decomposition trees, and watch the
system navigate them step by step.

---

### Phase 5 — Context Path Labeling and Tagging

**Goal:** Every completed traversal is recorded as a named, tagged
`ContextPath` and saved to `paths.json` (§6).

Deliverables:

- On session completion, build a `ContextPath` from the confirmed node sequence
- Generate a human-readable name from domain tag + pattern tag
  (e.g. `rust_ownership_violation`)
- Apply three-tier tag taxonomy: Domain / Pattern / Scope (§6.2)
- If an identical node sequence already exists, increment `usage_count`
  rather than creating a duplicate
- Write updated records to `paths.json`
- `--explain` output includes path name, full tag set, and usage count

Checkpoint:

```
$ chattie --explain "rust borrow error"

Path label:  rust_ownership_violation
Tags:        ownership, mutation, single_threaded, rust
Confidence:  0.75  |  Usage: 1  (just created)
```

After several runs:

```
Usage: 7
```

What this phase gives you: a persistent, human-readable record of every
reasoning route the system has ever taken. `paths.json` becomes a living
index of solved problem patterns you can read directly.

---

### Phase 6 — Path-Level Cache (Fast Re-resolution)

**Goal:** Queries whose active nodes match a known path bypass full
propagation and return the cached result immediately (§6.3).

Deliverables:

- On query start, compute overlap between active nodes and each known path
- If overlap $\geq$ 0.80, propose the path's solution directly with a
  `[cached path]` marker
- User can accept (`y`) or reject (`n`); rejection falls through to full
  propagation
- Cache hit does not update edge weights (no reinforcement for cache hits)
- Report cache hit rate in session summary

Checkpoint:

```
chattie> rust borrow error

[cached path match: rust_ownership_violation  overlap: 0.88]
Proposed: Only one mutable reference is permitted at a time.  [y/n]
> y
Answer confirmed from cache.
```

What this phase gives you: measurably faster repeated query resolution.
You can turn caching off with `--no-cache` to compare traversal vs cached
results and verify they agree.

---

### Phase 7 — Session Recording

**Goal:** Every session is persisted to `sessions.json` with the path
labels traversed, questions asked, and outcome (§6.4).

Deliverables:

- Assign each session a timestamped ID
- Record: input tokens, path labels traversed, breaking questions asked,
  branches taken, final outcome (`confirmed` / `rejected` / `abandoned`)
- `sessions.json` is append-only; never mutated retroactively
- CLI command `chattie --history` prints the last N sessions in summary form

Checkpoint:

```
$ chattie --history 3

2026-03-06-001  rust_ownership_violation       confirmed   questions: [ownership_dimension]
2026-03-06-002  tokio_deadlock_async           confirmed   questions: [concurrency_dimension]
2026-03-06-003  rust_lifetime_scope            rejected    questions: [ownership_dimension, lifetime_dimension]
```

What this phase gives you: a full audit trail. You can replay any session,
spot patterns in which questions are asked most often, and verify the system
is routing correctly over time.

---

### Phase 8 — Reinforcement Learning

**Goal:** Confirmed and rejected sessions update edge weights and path
confidence, so the graph gets better with use (§9).

Deliverables:

- On `confirmed`: apply positive reinforcement to each edge on the confirmed
  path — $w' = w + \alpha(1-w)$, $c' = c + \beta(1-c)$
- On `rejected`: apply negative reinforcement to the rejected path edges —
  $w' = w - \alpha w$, $c' = c - \beta c$
- Path-level reinforcement scaled by path length $n$: $\Delta w = \alpha / n$
- Stale path detection: mark paths with any edge below $\theta_c = 0.4$
  as stale; bypass cache for stale paths
- `--explain` shows before/after weight for each updated edge

Checkpoint:

After 5 confirmed sessions on `rust_ownership_violation`:

```
$ chattie --explain "borrow checker"

Edge borrow_checker → mutable_reference_conflict
  weight:  0.81 → 0.91  (reinforced ×5)
  confidence: 0.75 → 0.88
```

What this phase gives you: a graph that visibly improves. You can watch
`edges.json` evolve and confirm that heavily-used paths grow stronger.

---

### Phase 9 — Weak Answer Memory

**Goal:** Uncertain or incorrect answers are stored in `weak_memory.json`
and can be promoted to the main graph when corrected (§11).

Deliverables:

- When a session ends as `rejected` or `abandoned`, write a weak memory entry
  with the attempted path and solution
- CLI command `chattie --weak` lists all unresolved weak entries
- When a user provides a correction (`chattie --correct wm-0042 "ownership"`),
  the system promotes the corrected path and applies negative reinforcement
  to the incorrect one
- Resolved entries are archived (status set to `"resolved"`)

Checkpoint:

```
$ chattie --weak

wm-0042  [uncertain]  "rust borrow fail"  →  attempted: rust_lifetime_scope
wm-0051  [rejected]   "tokio hang"        →  attempted: thread_deadlock

$ chattie --correct wm-0042 "ownership"
Resolved: wm-0042
  + reinforced: rust_ownership_violation
  - penalized:  rust_lifetime_scope
```

What this phase gives you: a mechanism for the system to learn from its
own mistakes. The weak memory file is human-readable and correctable without
touching the graph directly.

---

### Phase 10 — Latent Node Discovery

**Goal:** The system automatically detects hidden shared concepts from
co-activation patterns and adds new `Latent` nodes to the graph (§10).

Deliverables:

- Track pairwise co-occurrence counters across sessions (persisted to
  `sessions.json`)
- After each session, compute similarity scores using the normalized
  co-occurrence formula (§10.2)
- When a group's pairwise similarity exceeds $\theta_L = 0.65$, create a
  `Latent` node with tag intersection and connecting edges at weight 0.5
- Flag new latent nodes for human review in `--explain` output
- CLI command `chattie --latent` lists all discovered latent nodes with their
  source groups

Checkpoint:

```
$ chattie --latent

deadlock  [latent]
  discovered from: tokio_deadlock, database_deadlock, thread_deadlock
  tags: [concurrency, lock, waiting]
  edges: tokio_runtime → deadlock (0.50)
         database_runtime → deadlock (0.50)
         thread_runtime → deadlock (0.50)
  status: pending review
```

What this phase gives you: emergent structure. The knowledge base grows on
its own as patterns repeat, and you can inspect every auto-created node
before it becomes load-bearing.

---

### Phase 11 — Automatic Context Expansion

**Goal:** Unknown tokens in queries cause provisional nodes to be created,
accumulate confidence through repeated use, and get promoted to active
nodes automatically (§15).

Deliverables:

- Unknown token → create provisional `Concept` node with `confidence: 0.1`,
  tagged as `unconfirmed`
- Each session that routes through the provisional node increments its
  confirmation counter
- At counter = 3: promote to active, assign tags from co-occurring confirmed
  nodes, add to matching paths
- `--explain` marks unconfirmed nodes with `[provisional]`
- `chattie --provisional` lists all pending nodes with confirmation counts

Checkpoint:

```
chattie> vectorization in rust

[new token] "vectorization" → provisional node created
[provisional] programming → vectorization (weight: 0.30, confidence: 0.10)
Answer: (low confidence — no confirmed path yet)

# After 3 queries through this node:
[promoted] vectorization → active (tags: ["rust", "performance"])
```

What this phase gives you: organic growth. The system learns vocabulary it
was never explicitly taught, and you can watch the promotion process in the
`--provisional` list.

---

### Phase 12 — Bias Tuning and Exploration Noise

**Goal:** Prevent heavily-reinforced paths from permanently drowning out
correct but less-used alternatives (§16).

Deliverables:

- Add exploration noise $\epsilon = 0.02$ to activation scores at propagation
  time: low-weight edges occasionally participate
- Add `--epsilon` flag to override the noise level at runtime
- Add a **bias audit**: `chattie --audit` shows the top 10 most dominant
  edges and flags any that have not been exercised in the last N sessions
  (configurable staleness window, default: 50 sessions)
- Stale-dominant edges receive a small passive decay per session
  ($w' = w \times 0.999$)

Checkpoint:

```
$ chattie --audit

Top dominant edges:
  borrow_checker → mutable_reference_conflict  weight: 0.93  last used: session 3  ✓ active
  lifetime        → lifetime_mismatch          weight: 0.88  last used: session 41  ✓ active
  mutex           → deadlock                   weight: 0.85  last used: session 12  ⚠ stale (>50 sessions)
  → passive decay applied
```

What this phase gives you: long-term graph health. The system stays
exploratory as it grows and doesn't permanently converge on a handful of
paths.

---

### Summary of Phase Deliverables

| Phase | Capability added                              | Inspectable artifact          |
| ----- | --------------------------------------------- | ----------------------------- |
| 0     | Compilable skeleton, file I/O                 | Binary runs, JSON layout      |
| 1     | Static keyword lookup from seed data          | Direct answers from seed      |
| 2     | Graph propagation with activation trace       | `--explain` hop-by-hop scores |
| 3     | Single yes/no clarification                   | Interactive question flow     |
| 4     | Multi-branch breaking questions               | Full decomposition tree       |
| 5     | Named path recording with tags                | `paths.json`                  |
| 6     | Path cache for fast re-resolution             | Cache hit/miss in output      |
| 7     | Session history with audit trail              | `sessions.json`, `--history`  |
| 8     | Reinforcement — graph improves with use       | Evolving `edges.json`         |
| 9     | Weak memory — mistakes stored and corrected   | `weak_memory.json`, `--weak`  |
| 10    | Latent node discovery                         | `--latent` review list        |
| 11    | Automatic context expansion                   | `--provisional` list          |
| 12    | Bias tuning, exploration noise, audit         | `--audit` report              |

---

### Phase 13 — Accelerator: BM25, N-grams, Session Context, Composite Answers

**Goal:** Close the perceptible gap between the graph system and an LLM on
bounded technical queries. Four targeted additions, each independent, each
approximately 20–150 lines of code against structures that already exist.
Can be applied in any order or individually.

---

#### 13.1 BM25 Retrieval over the Knowledge Base

**Fixes:** tokenizer failures on natural language queries; bad recall when
the user writes full sentences instead of keywords.

BM25 scores every node label and solution text against the full query using
term frequency and inverse document frequency. It replaces the exact token
match with a ranked activation seed list:

$$\text{BM25}(q, d) = \sum_{t \in q} \text{IDF}(t) \cdot \frac{f(t,d) \cdot (k_1 + 1)}{f(t,d) + k_1 \cdot (1 - b + b \cdot \frac{|d|}{\text{avgdl}})}$$

where $f(t,d)$ is term frequency in document $d$, $|d|$ is document length,
$k_1 = 1.2$ and $b = 0.75$ are standard defaults.

The top-K BM25 scores become activation seeds. The graph propagation step
(§4.3) is unchanged — only the seed generation improves.

```
Query: "why won't my rust code compile when I try to use two references"

BM25 seeds:
  mutable_reference_conflict   0.74
  borrow_checker               0.61
  lifetime_mismatch            0.38

→ same graph propagation as always, now with better seeds
```

Implementation: ~150 lines, no library dependency. The index is built once
at startup from all node labels and solution texts and held in memory.

---

#### 13.2 N-gram Token Matching

**Fixes:** multi-word concept matching. Currently `borrow checker`,
`stack overflow`, `null pointer`, and `type mismatch` only match if
the user writes exactly the node label as a single token.

Generate bigrams and trigrams from the input alongside unigrams and check
all of them against node labels:

```
Input: "borrow checker error"

Unigrams:  borrow, checker, error
Bigrams:   borrow_checker, checker_error
Trigrams:  (none long enough to match)

Matches:   borrow_checker  →  Node #2 [Concept]  ✓
           error           →  compile_error       ✓
```

Implementation: ~20 lines in the tokenizer. Works alongside BM25 — n-gram
hits can be used as exact-match seed boosts on top of BM25 scores.

---

#### 13.3 Session Context Carry-Forward

**Fixes:** multi-turn coherence. Currently each query starts cold — the
system has no memory of what was just discussed.

Keep the last 3 confirmed path labels in session state. At the start of
each new query, boost activation by a small constant $\delta = 0.2$ for
nodes on those paths:

```
Turn 1: "rust borrow error"  →  confirms rust_ownership_violation
         session context: [rust, borrow_checker, ownership]

Turn 2: "same problem but with threads"
         context boost applied:  borrow_checker  +0.20
                                 rust            +0.20
         concurrency_dimension triggers faster
         → no need to re-ask the ownership question
```

The carry-forward decays across turns: $\delta_n = 0.2 \times 0.6^{n-1}$
so context from 3 turns ago contributes almost nothing.

Implementation: ~50 lines. Session state record (§7) already exists;
carry-forward is a read of the last N entries at query start.

---

#### 13.4 Composite Answer Assembly

**Fixes:** the binary answer/question output. When two candidates are close
*and* related, returning both with a distinguishing hint is more useful
than asking a breaking question.

Trigger condition: top-2 candidates within $\theta_d$ **and** sharing at
least one tag (meaning they are related problems, not orthogonal ones).

```
Two related causes detected:

1. Multiple mutable references  (score: 0.74)
   Check if two &mut bindings exist in the same scope.
   → common in loops that hold a reference across an iteration

2. Lifetime mismatch  (score: 0.68)  
   Check if a borrow outlives its owner.
   → common when returning a reference from a function

Which matches your error message? [1/2/neither]
```

User answers `1`, `2`, or `neither` (falls through to breaking question).
This is the behavior users expect from a capable assistant. It also
generates richer session data — the user's selection tells the system
which of two related solutions was correct in this context.

Implementation: ~80 lines in the answer rendering step. The scores and
texts are already computed; this is purely a new output format path.

---

### Summary of Phase Deliverables

| Phase | Capability added                              | Inspectable artifact          |
| ----- | --------------------------------------------- | ----------------------------- |
| 0     | Compilable skeleton, file I/O                 | Binary runs, JSON layout      |
| 1     | Static keyword lookup from seed data          | Direct answers from seed      |
| 2     | Graph propagation with activation trace       | `--explain` hop-by-hop scores |
| 3     | Single yes/no clarification                   | Interactive question flow     |
| 4     | Multi-branch breaking questions               | Full decomposition tree       |
| 5     | Named path recording with tags                | `paths.json`                  |
| 6     | Path cache for fast re-resolution             | Cache hit/miss in output      |
| 7     | Session history with audit trail              | `sessions.json`, `--history`  |
| 8     | Reinforcement — graph improves with use       | Evolving `edges.json`         |
| 9     | Weak memory — mistakes stored and corrected   | `weak_memory.json`, `--weak`  |
| 10    | Latent node discovery                         | `--latent` review list        |
| 11    | Automatic context expansion                   | `--provisional` list          |
| 12    | Bias tuning, exploration noise, audit         | `--audit` report              |
| **13**| **BM25 + n-grams + context carry + composite**| **Near-LLM quality on domain queries** |

---

## 19. System Comparison

| Feature                   | This System        | LLM            |
| ------------------------- | ------------------ | -------------- |
| Compute requirement       | Tiny (<50 MB RAM)  | Huge (GB+)     |
| Learning method           | Incremental        | Full retraining|
| Explainability            | Full path trace    | Limited        |
| Deterministic             | Yes                | No             |
| Breaking question logic   | Explicit, labeled  | Implicit       |
| Path reuse / caching      | Named paths        | None           |
| Latent concept discovery  | Automatic          | Baked in       |
| Offline / air-gapped      | Yes                | Rarely         |

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

```
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

```
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

```
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

```
query + session_state  →  (answer | breaking_question) + updated_session_state
```

This maps directly onto a request/response API. Promotion to a network
service requires a thin layer, not a redesign.

**Architectural path:**

```
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

```
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

```
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
