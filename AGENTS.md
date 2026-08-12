# AGENTS.md — Engram

**L0 identity for AI-assisted development.** Human-authored and reviewed; treat as the map,
not the terrain. Read the roadmap and the latest handover before starting work.

## What Engram is

A deterministic reasoning kernel: a CLI application that navigates a weighted context graph
to answer bounded-domain questions, decomposes ambiguity with labelled breaking questions,
and learns incrementally from confirmed sessions. Single binary, <100 MB memory, no GPU, no
runtime model dependency. Determinism is a hard invariant: same input + same graph state →
same output, byte for byte.

## Repository layout

```text
app/            Rust crate (the binary + knowledge data)
docs/           Design documentation (proposal, architecture, roadmap, metrics, …)
handover/       Iteration logs — one per task, committed (the project's memory)
.github/        CI workflow + funding config
```

## Development workflow

Development follows a **roadmap → task → handoff** methodology (ICM/MWP):

1. **Accept** — read `docs/roadmap.md` (§18.0 status table; the current phase is the contract),
   read the latest file in `handover/`, state assumptions and a plan with verification steps.
2. **Process** — work only on the current task, on the current phase's dev branch. Verify as
   you go. If scope must change, pause and say so.
3. **Handoff** — flip the roadmap status, write the handover log, commit both with the
   delivery commit. A session without a handover is a lost session.

## Engineering standards

The project follows the engineering workspace guidelines by name:

- **Rust development** — idiomatic Rust, minimal dependencies, production-grade error types,
  rustdoc on every module and public item, unit and integration tests.
- **Software development style** — doc comments on every module and function; tests are part
  of the deliverable; pure core, effects at the edges; surgical edits; simplicity first.
- **JigsawFlow pattern + singleton-registry** — capability contracts, facades, and graceful
  degradation. Adoption is gated on the capability layer (see roadmap backlog C11–C13); do
  not introduce the registry before that gate.
- **Git commit conventions** — `feat (NN)` / `fix (NN)` / `docs` / `chore`, where `NN` is the
  roadmap story (phase) number; drop `(NN)` when no story applies.
- **Platform** — offline-first, computational sovereignty: no mandatory network, no telemetry,
  deployable on edge/SBC hardware.

## Branch policy

- `main` — stable, CI-green only.
- One dev branch per roadmap phase (`phase-3/clarification-questions`, …). Feature work
  happens there; merges to `main` only when the phase gate passes (CI green, tests, handover).

## Verification gate (runs on every change)

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

## Determinism rules

- No `HashMap` (or any order-nondeterministic collection) on the reasoning path — `BTreeMap`
  or explicitly sorted iteration only; f32 summation order must be fixed.
- Ties resolve by node id, never by iteration order.
- The determinism regression test (`engine.rs` tests) must stay green.
