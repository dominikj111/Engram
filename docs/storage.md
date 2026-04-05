# Engram — CLI Behavior and Memory Layout

*Part of the [Engram design documentation](proposal.md).*

---

## 12. CLI Behavior

### 12.1 Interactive Loop

```text
$ engram

engram> why rust borrow error?

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

```text
$ engram "why rust borrow error?"

Possible cause:  multiple mutable references (score: 0.82)
Path:            rust_ownership_violation
Confidence:      0.75
```

### 12.3 Explanation Mode

```text
$ engram --explain "why rust borrow error?"

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

```text
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
