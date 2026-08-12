# Handover — 2026-08-12 — Phase 2 engine merge

| Section | Contents |
|---|---|
| **Task** | Roadmap Phase 2 "Graph Activation and Propagation". Status before: Pending (work existed on closed PR #1 branch). Status after: **Done** on `main`. |

## What was done

- **Merged the Phase 2 engine from the closed PR #1** (Jules, `multi-hop-engine-...`
  branch) into `main` as `feat (2)` (merge commit `637d494`). Work preserved:
  - `app/src/engine.rs` — Engine module: tiered activation seeding (label 0.90 /
    label-part 0.70 / tag 0.45), multi-hop propagation with decay λ=0.85, max 4 hops,
    activation summation, `ConfidenceLevel` (High/Medium/Low/Unknown) from
    θ_a=0.75 and θ_d=0.15, `--explain` trace output.
  - `main.rs` refactored to delegate query handling to the engine.
  - `model.rs` gained `ConfidenceLevel`.
  - Seed knowledge extended with multi-hop test chains (nodes 12–14: browser/network/
    timeout context).
- **Fixed a determinism violation found in the merged code** (`fix (2)` `b63a047`):
  activations used `std::collections::HashMap`, which iterates in random per-process
  order (RandomState) — f32 summation order and tie ranking could differ between runs.
  Replaced with `BTreeMap` (deterministic iteration + summation order), added an explicit
  score tie-break by node id, and added two engine tests.
- **Normalised crate formatting** (`cargo fmt`) — `main.rs`, `knowledge.rs`, `engine.rs`.
- **Branch cleanup:** deleted local `pr1`, remote
  `multi-hop-engine-4789464145010592537` and `phase-2/graph-activation-propagation`.
  Created `phase-3/clarification-questions` at `main` (pushed).
- **Roadmap updated** (`docs/roadmap.md`): added §18.0 status table, engineering
  compliance backlog (C1–C10), and per-phase acceptance criteria.

## What was done differently

- The closed PR was **re-opened in effect by merging the commit directly** — the work was
  valuable (it *is* Phase 2) and applied cleanly (main had only doc/NOTICE changes since
  the branch point). Follow-up hardening (tests, CI) is deferred to the backlog instead of
  blocking the merge, since the repo currently has no test infrastructure at all.
- The determinism fix was applied immediately rather than listed as backlog — it violates
  the workspace invariant #1 and would have been wrong to ship.

## Verification

- `cargo build`, `cargo test` (2 tests pass), `cargo clippy --all-targets` (0 warnings),
  `cargo fmt --check` (clean).
- Cross-process determinism: 10 separate runs of `engram --explain "error"` (a query that
  produces exactly tied solution scores) → 1 unique md5 across all runs.

## Open questions

- None blocking. Recommend deciding C9 (remove the `--audit` stub until Phase 12) during
  the next task.

## Next step

- Start roadmap **Phase 3 — Clarification Questions (single-branch)** on
  `phase-3/clarification-questions`, satisfying the phase acceptance criteria in
  roadmap §18.0 (tests, rustdoc, CI, determinism guard, handover).
- Suggested order within Phase 3: C2 (CI) + C4 (rustdoc) first so every subsequent phase
  ships through the same gate.
