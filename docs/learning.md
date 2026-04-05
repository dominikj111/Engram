# Engram — Learning, Memory, and User Profiles

*Part of the [Engram design documentation](proposal.md).*

---

## 8. Learning from Interaction

Every completed session updates the graph along the confirmed path.

Example:

```text
Q: Rust borrow error
Bot: [breaking] Are multiple parts of the code mutably borrowing the same value?
User: yes
Bot: Only one mutable reference is permitted at a time in the same scope.
User: confirmed — that solved it
```

Graph updates applied:

```text
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

### 9.2.1 Confidence as a Separate Dimension

`weight` and `confidence` serve different purposes and must not be conflated:

- **weight** — encodes path frequency: how often this edge was traversed
- **confidence** — encodes reliability: `f(sample_size, variance)` over outcomes

A high-weight edge reached by many sessions with inconsistent outcomes should
have low confidence. A low-weight edge reached by few sessions with unanimous
confirmation should have high confidence.

Confidence is computed as:

$$c = \frac{n_{\text{confirmed}}}{n_{\text{confirmed}} + n_{\text{rejected}} + 1} \cdot \left(1 - \frac{1}{1 + n_{\text{total}}}\right)$$

The second factor suppresses confidence on edges with very few sessions,
preventing early wrong reinforcement from locking in permanent bias.

Additionally, edge confidence decays slowly over time when no new sessions
traverse it — preventing stale high-confidence paths from dominating a domain
that has evolved:

$$c' = c \cdot \delta \quad \text{(applied periodically, default } \delta = 0.99 \text{)}$$

Weight does not decay — frequency is a historical fact. Only confidence decays,
reflecting reduced certainty about correctness when a path has not been
recently validated.

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

Before creation, the system computes a **predictive gain** for the candidate
latent node: would introducing $L$ reduce the average number of breaking
questions needed to reach a solution along the affected paths? If
$\Delta Q_{\text{avg}} \leq 0$ — the node adds no disambiguation value — it
is rejected regardless of co-occurrence score. This prevents graph bloat from
nodes that capture correlation without improving routing.

1. Create a new `Latent` node $L$ with an auto-generated label
2. Add edges $A \to L$, $B \to L$, $C \to L$ with initial weight $0.5$
3. Tag $L$ with the intersection of the tag sets of $A$, $B$, $C$
4. Label the new node for human review (surfaced in explanation mode)

Example:

```text
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

### 11.1.1 Expiry Policy

Weak memory entries that are never corrected become noise. Entries expire
under two conditions:

- **Age expiry:** entry has not been accessed or updated in 90 days
- **Supersession:** a high-confidence path now covers the same token set that
  triggered the weak answer — the graph has learned what the weak memory was
  waiting to teach

Expired entries move to status `"expired"` and are excluded from matching.
They are retained in the file for audit purposes but do not participate in
promotion or reinforcement.

Similar failure patterns are clustered before expiry: if five expired entries
share the same attempted path, that path receives a mild negative reinforcement
signal even without an explicit user correction.

### 11.2 Promotion to Main Graph

When a user later provides the correct answer:

```text
User: Actually the issue was mutable reference conflict
```

The system:

1. Locates the weak memory entry by session or question hash
2. Resolves the correct path ("rust\_ownership\_violation")
3. Applies positive reinforcement to the correct path
4. Applies negative reinforcement to the incorrect path
5. Updates the entry status to `"resolved"` and archives it

---

## 11.3 UI Context Memory

The session already records reasoning context — which paths were traversed,
which questions were answered. But the interface layer has its own memory
requirement: what was shown to the user, what they clicked, and what options
were presented. Without this, the UI becomes inconsistent across turns.

```rust
struct UIContextRecord {
    turn:       u32,
    components: Vec<UIComponent>,  // what was rendered in this turn
    selection:  Option<String>,    // what the user clicked or said, if anything
    dismissed:  Vec<String>,       // options presented but not chosen
}
```

The session record (§7) is extended with a `ui_history: Vec<UIContextRecord>`.
This enables several behaviours that are otherwise impossible:

**No repeated options.** If the user dismissed "Reboot router" in turn 2, it
is not offered again in turn 4 unless a new activation path explicitly
re-introduces it.

**Coherent multi-turn forms.** If a parameter collection form was partially
filled in turn 3, the system pre-populates it with already-confirmed values
in turn 5 rather than starting blank.

**Audit trail for UI actions.** Every button click and form submission is
logged alongside the reasoning trace — essential for debugging interactions
where the user claims "I already tried that."

UI context records are held in memory for the session duration and flushed
to `sessions.json` on session close. They are not persisted between sessions;
the reasoning context already captures what mattered.

---

## 11.5 User Profile

The system maintains a lightweight per-user profile derived from accumulated
session data. No personal data is stored — the profile is a statistical
summary of reasoning patterns observed across sessions.

```rust
struct UserProfile {
    id:               String,                   // hashed OS username or auth token
    dimension_counts: HashMap<String, u32>,     // breaking questions asked per dimension
    confirmed_paths:  HashMap<String, u32>,     // path label → confirmation count
    skill_level:      SkillLevel,               // derived from session history
    last_active:      Timestamp,
}

enum SkillLevel { Novice, Intermediate, Expert }
```

### 11.5.1 Skill Level Derivation

`SkillLevel` is derived from two signals: average breaking questions asked per
session and total confirmed paths. Both signals update incrementally.

| Avg questions/session | Confirmed paths | Skill Level  |
| --------------------- | --------------- | ------------ |
| ≥ 2.5                 | < 10            | Novice       |
| 1.0 – 2.5             | 10 – 50         | Intermediate |
| < 1.0                 | > 50            | Expert       |

### 11.5.2 Profile-Driven Routing

The profile modifies system behavior in two ways:

**Breaking question selection:** dimensions the user has resolved correctly
multiple times are de-prioritised. The system proposes the high-confidence
branch directly rather than re-asking a settled dimension:

```text
User has confirmed ownership_dimension correctly 8 times.
→ Skip ownership_dimension breaking question
→ Propose rust_ownership_violation directly with [profile shortcut] marker
→ User can override with 'n' to force the full question flow
```

**Response verbosity:** Novice users receive full explanation traces by
default; Expert users receive terse single-line answers unless `--explain`
is passed explicitly:

```text
Novice:
  Answer: Only one mutable reference is permitted at a time in the same scope.
  Path:   rust_ownership_violation
  Why:    borrow_checker → mutable_reference_conflict (score 0.91)

Expert:
  mutable reference conflict  [rust_ownership_violation  0.91]
```

User profiles are stored in `profiles.json`. For CLI use, the profile ID is
derived from the OS username. For network deployments (§20.8), it maps to
the authenticated user identifier, enabling cross-channel profile continuity.
