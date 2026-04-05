# Engram — Knowledge Base, Context Expansion, and Context Bias

*Part of the [Engram design documentation](proposal.md).*

---

## 14. Initial Knowledge Base

Start with a curated seed of 100–300 nodes covering common problem patterns.

**Seed concepts:**

```text
rust, borrow_checker, mutable_reference, lifetime, compile_error,
tokio, async, deadlock, mutex, thread, ownership, trait, generic,
type_error, null_pointer, index_out_of_bounds, stack_overflow
```

**Seed solutions:**

```text
mutable_reference_conflict:  "Only one mutable reference is permitted at a time."
lifetime_issue:              "Check that all borrows are within the owner's scope."
deadlock:                    "Ensure lock acquisition order is consistent across threads."
```

**Seed breaking questions:**

```text
ownership_dimension:
  prompt: "Are multiple parts of the code mutably borrowing the same value?"
  branches: yes → mutable_reference_conflict | no → lifetime_mismatch

concurrency_dimension:
  prompt: "Does the error occur only under concurrent execution?"
  branches: yes → deadlock | no → single_thread_error
```

**Seed paths:**

```text
rust_ownership_violation:  [rust → borrow_checker → mutable_reference_conflict]
  tags: [ownership, mutation, single_threaded, rust]

rust_lifetime_scope:       [rust → borrow_checker → lifetime_mismatch]
  tags: [lifetime, scope, rust]
```

---

## 15. Automatic Context Expansion

When a query contains an unrecognized token, the system creates a provisional node:

```text
Token:    "vectorization"
Action:   create Concept node "vectorization", tags: []
Connect:  programming → vectorization  (weight: 0.3, confidence: 0.1)
```

The node is flagged as **unconfirmed**. After three interactions that route
through it, confidence rises above the confirmation threshold and the node
is promoted to fully active. Tags and path memberships are assigned during
the promotion step.

---

## 15.5 Real-World Noise Handling

The system currently assumes reasonably clean, cooperative input. Real users
write vaguely, emotionally, with typos, and with partial information. These
four conditions require explicit handling — not as special cases, but as
first-class activation paths.

### 15.5.1 Partial Activation

When fewer than the minimum threshold of tokens match known nodes, full
graph propagation is wasteful. Instead, the system enters a
**best-guess + correction loop**:

1. Activate whatever nodes matched, even at low confidence
2. Surface the top candidate with a `[Low]` confidence badge (§4.5)
3. Ask a single targeted clarification rather than a full breaking question
4. If confirmed, reinforce; if rejected, record as weak memory and request
   the user to rephrase

This is how the `Unknown` confidence state (§4.5) resolves — not by saying
"I don't understand", but by making the best available guess visible and
correctable.

### 15.5.2 Fuzzy Token Matching

Before the provisional node creation path (§15) is triggered, the tokenizer
applies three fuzzy layers in sequence:

```text
Input token: "conection" (typo)

1. Edit-distance check (Levenshtein ≤ 2):
   → "connection" matches node connectivity_issue  ✓

Input token: "keeps cutting out"

2. N-gram match (§13.2 bigrams):
   → "cutting_out" → no direct match
   → fallback to BM25 (§13.1) over solution texts
   → scores connectivity_issue 0.61  ✓

Input token: "AAARGH internet broken again"

3. Emotional language strip:
   → strip stop words and intensifiers
   → retain: ["internet", "broken"]
   → normal activation proceeds
```

The fuzzy layers run before any graph activation. The graph never sees
malformed input — only normalised tokens.

### 15.5.3 Incomplete Information Tolerance

When required parameters for an action contract cannot be resolved (§3.4),
the system does not fail — it starts execution with the information it has
and defers the missing fields:

```text
Action: CheckLineStatus
  account_id → resolved from session
  postcode   → not found anywhere

→ Initiate partial resolution:
   "I can check your line status — what's your postcode?"
   [one targeted question, not a form dump]
```

The action contract tracks which parameters are deferred. If the user
answers, the execution proceeds. If they do not (e.g. voice drop-off,
timeout), the session is marked `Abandoned` with the partial context saved.

---

## 16. Context Bias

Frequently reinforced paths become dominant. High-weight edges are traversed
first during propagation, effectively pre-selecting the most probable solution
before the full graph is evaluated.

```text
borrow_checker → mutable_reference_conflict
  weight: 0.93   ← heavily reinforced
```

This is analogous to **learned bias in a neural network**: the system develops
a prior shaped by historical interactions.

To prevent over-bias locking out correct but rare paths, a small **exploration
noise** $\epsilon$ (default 0.02) is added to activation scores, ensuring
lower-weight edges occasionally participate in propagation.
