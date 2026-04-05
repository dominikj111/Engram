# Engram — Development Roadmap

*Part of the [Engram design documentation](proposal.md).*

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
- `engram` binary runs, prints a greeting, and exits cleanly

Checkpoint:

```text
$ engram
engram v0.1 — knowledge loaded: 0 nodes, 0 edges
engram>
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

```text
$ engram "mutable reference"
Answer: Only one mutable reference is permitted at a time in the same scope.
Path:   direct match → mutable_reference_conflict

$ engram --explain "mutable reference"
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

```text
$ engram --explain "why rust borrow error"

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

```text
engram> why rust borrow error

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

```text
engram> deadlock in tokio

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

```text
$ engram --explain "rust borrow error"

Path label:  rust_ownership_violation
Tags:        ownership, mutation, single_threaded, rust
Confidence:  0.75  |  Usage: 1  (just created)
```

After several runs:

```text
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

```text
engram> rust borrow error

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
- CLI command `engram --history` prints the last N sessions in summary form

Checkpoint:

```text
$ engram --history 3

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

```text
$ engram --explain "borrow checker"

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
- CLI command `engram --weak` lists all unresolved weak entries
- When a user provides a correction (`engram --correct wm-0042 "ownership"`),
  the system promotes the corrected path and applies negative reinforcement
  to the incorrect one
- Resolved entries are archived (status set to `"resolved"`)

Checkpoint:

```text
$ engram --weak

wm-0042  [uncertain]  "rust borrow fail"  →  attempted: rust_lifetime_scope
wm-0051  [rejected]   "tokio hang"        →  attempted: thread_deadlock

$ engram --correct wm-0042 "ownership"
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
- CLI command `engram --latent` lists all discovered latent nodes with their
  source groups

Checkpoint:

```text
$ engram --latent

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
- `engram --provisional` lists all pending nodes with confirmation counts

Checkpoint:

```text
engram> vectorization in rust

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
- Add a **bias audit**: `engram --audit` shows the top 10 most dominant
  edges and flags any that have not been exercised in the last N sessions
  (configurable staleness window, default: 50 sessions)
- Stale-dominant edges receive a small passive decay per session
  ($w' = w \times 0.999$)

Checkpoint:

```text
$ engram --audit

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

```text
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

```text
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

```text
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

```text
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
