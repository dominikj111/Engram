# Contributing to Engram

Thank you for your interest in contributing. This document explains how the
project is structured and what kinds of contributions are welcome.

---

## The relationship between docs and code

The design documents in [docs/](docs/) are the **specification**. The Rust
code in [app/](app/) is the **implementation** of that specification.

This distinction matters for contributions:

| Contribution type | Welcome? | Process |
| --- | --- | --- |
| Code implementing a documented phase | Yes | Standard PR |
| Bug fixes and correctness improvements | Yes | Standard PR |
| Tests, benchmarks, examples | Yes | Standard PR |
| Refactoring within spec | Yes | Standard PR |
| New seed knowledge / domain graphs | Yes | Standard PR |
| Spec clarifications (typos, ambiguous wording) | Yes | Standard PR |
| Design changes to docs/ | Rarely | Discussion first — see below |

---

## Code contributions

All implementation must follow the design specified in [docs/proposal.md](docs/proposal.md)
and its linked documents. The roadmap phases in [docs/roadmap.md](docs/roadmap.md)
define the intended sequence.

Before opening a PR:

- Check which roadmap phase the work belongs to
- Ensure the implementation matches the data structures in [docs/architecture.md](docs/architecture.md)
- Keep the determinism guarantee: same input must always produce the same reasoning path

---

## Design document changes

The documents in [docs/](docs/) represent deliberate architectural decisions.
They are not freely editable.

A design change is appropriate only when:

1. A documented behaviour is provably incorrect or creates an unsolvable implementation problem
2. An industry requirement or real-world deployment context reveals a genuine gap in the spec
3. A proposed improvement is mathematically sound and consistent with the existing formulas and principles

**Process for proposing a design change:**

1. Open a GitHub issue describing the problem, not the solution
2. Include the specific section and the reason the current spec is insufficient
3. Wait for discussion and explicit maintainer approval before writing anything
4. If approved, the doc change and any corresponding code change land in the same PR

Changes that reframe the architecture, add new phases, or alter core formulas
(activation propagation, reinforcement rules, latent node discovery) require
a higher bar of justification and will be reviewed carefully.

---

## What is out of scope

- Adding LLM or neural network dependencies to the core engine
- Making the system non-deterministic
- Removing the offline-first constraint
- Changing the graph-first architecture to retrieval-based or generative

These are not limitations to work around — they are the design.

---

## Getting started

```sh
git clone https://github.com/dominikj111/engram.git
cd engram/app
cargo build
cargo test
```

Read [docs/roadmap.md](docs/roadmap.md) to understand which phase is current
and what the next milestone looks like.
