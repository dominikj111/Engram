# Handover — Engram stage-3 ideation inputs (2026-09-03)

Next Engram session starting point. Source drop: `taiga.txt` item 4 + duck.ai
exploration transcript (Mistral Small 4) — raw in
`inbox/processed/2026-09-03-todo-drop/duck.ai_2026-09-03_13-41-07.txt`.

## The task (owner's words)

"Engram, evaluate current state, other branch to emerge and suggest next changes to move
to stage 3."

Roadmap state (`docs/roadmap.md`): phases 0–2 **done** on `main` (skeleton/file I/O,
static keyword lookup, graph propagation with activation trace — PR #1 merged). Phase 3
= Clarification Questions (single-branch) is the next pending phase.

## Direction material from the exploration (duck.ai, Mistral — ideas, not decisions)

Owner's working model (second prompt is the accurate one):
- Engram doesn't store exact prompts/data — it sets **internal weights**, likely encoding
  answers only → **security by design**: user inputs are not stored.
- It should be sensitive to *some* prompt parts ("anchors"/labels), react/learn accordingly.
- Outputs are more generic — human-readable wiki-style knowledge, business-logic
  resolution for coding questions.
- Offline use remains a valid single use case; user feedback adjusts the encoded answers
  over time.

Model's gaps list (what to watch): deterministic-vs-probabilistic output, anchor
definition/consistency, ambiguity handling, feedback attribution/versioning,
offline-vs-online learning tension, scalability of a locked-answer store. Model's
suggested architecture (embedding matching, constrained decoding, draft→locked answer
promotion, delta updates) is generic LLM-ecosystem boilerplate — evaluate against
Engram's actual graph-activation design before adopting anything.

## Suggested next-session flow

1. Read `docs/roadmap.md` phases 3+ and `docs/future.md`/`docs/learning.md` framing.
2. Audit where the anchors/weights idea fits (or conflicts with) the existing engine
   (activation trace, path reuse, deterministic propagation).
3. Propose the branch + concrete phase-3 changes in the roadmap.
