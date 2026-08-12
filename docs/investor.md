# Engram — Experiment Roadmap & Post-Validation Investor Memo

> Status: solo developer (UK sole trader). A Rust CLI implementation exists in the repo
> (`app/src`, Apache-2.0); the design docs are complete (12 documents). **Experiments come
> first.** The developer will not seek investors before the gates pass and proofs are public.
> This memo contains: (A) the experiment roadmap (~9–12 months, self-funded), (B) the
> post-validation production phase and funding math, (C) investor answers, (D) outreach
> channels.
>
> Source: `docs/proposal.md`, `docs/architecture.md`, `docs/roadmap.md`, `docs/metrics.md`,
> `docs/use_cases.md`, `docs/future.md` (+ index).

---

## The ask at a glance

| | |
|---|---|
| **Stage** | Post-validation pre-seed — money requested only after the cost-optimizer gate passes (use_cases §8 roadmap, Stage 3 + Stage 6) |
| **Ask** | **£40,000** (€46,809 / $54,022) — 6 months, one fixed number |
| **Use of funds** | £30k founder runway at **half market rate** (the founder haircut) + £10k business budget (compute, LLM tokens, tools, legal, community) |
| **Tranches** | 2 × £20k, milestone-gated; each tranche releases only when its gate passes |
| **Instrument** | SAFE / convertible note; quarterly demo + metric report; direct investor access to the reasoning traces (auditable by design) |
| **Why now** | The critical risk — *does a deterministic graph actually resolve a bounded domain without an LLM?* — is resolved by the zero-API-call proof; the price is still small |
| **Moat** | The graph accumulates confirmed sessions — switching cost compounds automatically (use_cases §8) |
| **Follow-on** | Only after first users: the LLM agent mesh / cost-optimizer service, MCP knowledge server, tool-security boundary (§8/§9/§11) |

---

## 1. Development profile — solo developer (UK sole trader)

- **Role:** single developer, UK self-employed (sole trader / self-employed).
- **Income benchmark:** Principal Architect / Staff Software Developer, UK market — **£90k
  gross/year minimum, £150k at the top end**. This is the opportunity cost of the developer's
  time and the basis for every funding number below.
- **Tax note (sole trader):** income is invoiced **gross**; the developer settles own income tax
  and Class 4 National Insurance on profits. Allowable business expenses (hardware, compute,
  LLM tokens, tools, software, home office) reduce taxable profit. No employer-side PAYE/NIC
  split, no office overhead. Figures are **gross**; net depends on the individual tax position
  (not tax advice — see an accountant).

### 1.1 Income benchmark in three currencies

Rates: £1 ≈ €1.17, £1 ≈ $1.35 (approximate, [date of writing]; check current rates before use).

| Period | GBP | EUR | USD |
|---|---|---|---|
| Annual, low | £90,000 | €105,320 | $121,549 |
| Annual, high | £150,000 | €175,533 | $202,582 |
| Monthly, low | £7,500 | €8,777 | $10,129 |
| Monthly, high | £12,500 | €14,628 | $16,882 |
| **Committed founder runway — basis of the ask** | **£5,000 / mo** | **€5,851 / mo** | **$6,753 / mo** |
| **…annual equivalent (half market rate)** | **£60,000** | **€70,213** | **$81,033** |
| **…day rate (220 days)** | **£273** | **€319** | **$368** |

**Committed rate, not a range:** the benchmark above is the developer's UK market rate
(£90k–150k). The ask is based on a **founder runway of £5,000/month gross — half the market
rate**. The other half is the **founder haircut**: the developer invests that difference in the
venture in exchange for equity — the standard founder deal, and the strongest commitment signal
a solo founder can send. Every funding figure below is derived from this single fixed rate.

---

## 2. Experiment roadmap — first priority (~9–12 months, self-funded)

**Principle:** the developer funds the core and the experiments from freelance income. No
investor outreach happens before the gates pass and proofs are public. Each experiment has
mechanical pass criteria; further work — not investor money — is released only when the
previous gate passes.

The roadmap phases (docs/roadmap.md §18, phases 0–15) are small and explicitly ordered so the
system is **useful from Phase 2 onward**. A Rust implementation exists already; the experiments
below are the gates that turn the design into proven behaviour.

### Experiment 1 — Phases 0–2: skeleton, seed knowledge, graph activation
**Build/verify:** compilable binary, `knowledge/` JSON loading, tokenizer; activation
propagation (`a_target = a_source × w × λ`, default 4 hops); top-ranked solution above
θ_a = 0.75.

**Pass criteria (roadmap §18 Phase 2):**
- `engram --explain "why rust borrow error"` prints a hop-by-hop activation trace with scores.
- The top solution is returned only when its score exceeds the threshold; otherwise the system
  enters clarification.

**Time:** ~1–2 months (code base exists). **Public proof:** the `--explain` activation trace.

### Experiment 2 — Phases 3–5: breaking questions + context path labeling
**Build:** single-branch clarification, then multi-branch breaking questions (θ_d = 0.15,
max 3 per session); named, tagged `ContextPath`s written to `paths.json` (Domain/Pattern/Scope
tag taxonomy).

**Pass criteria:** an ambiguous query is decomposed through labeled branches; the confirmed path
is recorded with a human-readable name and usage count; `--explain` shows which branch
eliminated which candidate.

**Time:** ~1–2 months.

### Experiment 3 — Phases 6–9 + metrics: learning loop — **THE GATE**
**Build:** path-level cache; session recording (`sessions.json`, append-only audit trail);
reinforcement learning (positive/negative weight updates, stale-path detection); weak answer
memory (`weak_memory.json`, `--weak`, `--correct`); the `engram --metrics` command.

**Pass criteria — the cost-optimizer proof (use_cases §8 roadmap, Stage 3):**
> *A query handled entirely by Engram — full reasoning trace, zero API calls, measurable
> confidence score.*

plus the production metrics (docs/metrics.md §17.5) on a seed domain:

| Metric | Target |
|---|---|
| Resolution rate | > 80% |
| Time to resolution | < 4 turns |
| Questions per resolution (primary learning KPI) | trending ↓ |
| Escalation rate | < 15% |
| Correction rate | < 10% |
| Cache hit rate (Phase 6+) | > 40% |

**Time:** ~2–3 months. **This is the gate:** it converts "deterministic graph" into
"measurably resolves a bounded domain with zero LLM cost".

### Experiment 4 — Phase 13 accelerator + one real specialist domain
**Build:** BM25 retrieval, n-gram matching, session context carry, composite answers (each
~20–150 lines against existing structures, roadmap §18 Phase 13); then seed one narrow
specialist domain (e.g. CI/CD triage or structured log analysis) per the use_cases §8 roadmap
Stage 3–6.

**Pass criteria:** in the specialist domain, the measured split approaches the design profile —
~70–80% of queries handled by the graph alone (microseconds, zero API cost), ~15–25%
preprocessed with reduced tokens, ~5% genuinely novel (use_cases §8.4). Questions per
resolution shows a negative slope over a rolling window of sessions.

**Time:** ~2–3 months.

### Timeline (self-funded)

```
M0–2     Experiment 1 — activation + --explain
M2–4     Experiment 2 — breaking questions + path labeling
M4–7     Experiment 3 — learning loop + metrics   ← THE GATE: zero-API-call proof + metrics
M7–10    Experiment 4 — BM25 accelerator + one specialist domain
         ▼
         GATE: a bounded-domain query resolved entirely by the graph, zero API calls,
         with the §17.5 metrics; proofs public — only now are investors approached
```

**Self-funding during this phase:** freelance income covers living costs; project costs are
tiny (single binary, no GPU, no API dependency by design — the system's whole point).

---

## 3. Post-validation: production phase (~6 months, investor-funded)

Once the gate passes, the first product follows the stated strategic priority (use_cases
"Strategic Priority"): the **LLM agent mesh / cost optimizer** — *reduce LLM API spend by
70–80% in bounded domains*. The narrative is CFO-legible, positions Engram *with* the AI
ecosystem (LLM as a teaching signal, not a dependency), and the switching cost compounds as the
graph accumulates confirmed sessions. Adjacent first-class products from the same engine: the
**MCP server** (Engram as a confidence-weighted knowledge database for LLM agents, §9) and the
**LLM tool security boundary** (structural policy enforcement — enumerable actions, not prompt
guardrails, §11).

### 3.1 Funding math — one fixed ask, half-rate runway, business budget

| Phase | Duration | Ask (GBP) | EUR | USD |
|---|---|---|---|---|
| **Initial ask — production MVP** | **6 months** | **£40,000** | **€46,809** | **$54,022** |
| Extended: production + first users | 12 months | £80,000 | €93,618 | $108,044 |
| Corridor (scope variance only) | 6 months | £37k–43k | €43k–50k | $50k–58k |

**Use of funds:**

| Line | GBP | EUR | USD |
|---|---|---|---|
| Founder runway — 6 months @ £5k/mo (**half market rate**) | £30,000 | €35,107 | $40,516 |
| Business budget: compute, tooling, seed-domain knowledge authoring, legal, community | £10,000 | €11,702 | $13,505 |
| **Total** | **£40,000** | **€46,809** | **$54,022** |

**Fixed value, narrow corridor.** The ask is a single number — **£40,000** — not a range. The
±£3k corridor exists only for final scope variance, never for the rate. Tranches:

| Tranche | Releases on | GBP | EUR | USD |
|---|---|---|---|---|
| 1 | start of production phase | £20,000 | €23,404 | $27,011 |
| 2 | cost-optimizer MVP acceptance: one paying-shaped domain at the §8.4 cost profile with the §17.5 metrics | £20,000 | €23,404 | $27,011 |

- **What the money buys:** 6 months of founder time at half market rate + a £10k business
  budget. The money funds the *venture*, not a lifestyle.
- **Why this is enough:** lean by design — <100 MB memory, single binary, no GPU, no runtime
  model dependency (docs/proposal.md §1.2); the experiments already proved the learning loop.
- **Why it is still below €250k+:** a classic seed funds hiring and burn; here the milestone is
  a working cost-optimizer deployment with measured savings, and the follow-on round comes with
  real customer numbers.
- **Structure:** SAFE or convertible note; quarterly demo + metric report; investor access to
  the reasoning traces (auditable by design).
- **Follow-on (only after first users):** hosted agent-mesh service, MCP knowledge server,
  tool-security product (§8/§9/§11); later the hierarchical distributed aggregation (§7).

---

## 4. Investor answers

### 4.1 The problem
LLM agents re-reason from scratch on every query: cold starts, full-context reads, token and
GPU burn — even for problems the system has resolved hundreds of times. Guardrails are prompts
(bypassable) or scattered runtime checks (fragile). And every query costs money. For bounded
domains, 200–2000 recurring problem signatures account for 80–95% of queries (future.md §19.1)
— a learned graph resolves those in microseconds, offline, for zero API cost.

### 4.2 Why now
LLM spend is a top enterprise complaint; agent frameworks are commoditising; MCP is becoming
the standard tool interface; and every provider pushes more tokens through more agents. A
deterministic, auditable layer that *sits in front of* the LLM and routes the repetitive 80%
away is aligned with the ecosystem rather than against it (use_cases "Strategic Priority").

### 4.3 Solution and differentiation

| Engram | LLM / agent frameworks |
|---|---|
| Deterministic graph traversal — same input, same graph, same output | Stochastic by design |
| Full reasoning trace, every node and edge named | Reasoning opaque; tool-call log only |
| <100 MB, single CPU, offline, no API key | GB+ weights, GPU/API required |
| Learns incrementally from sessions — no retraining | Full retraining / fine-tuning |
| Stores patterns, never raw content (structural privacy) | Conversation history retained |
| Structural policy boundary — actions enumerable in `actions.json` | Prompt-level guardrails |
| Guaranteed termination — bounded FSM, not Turing-complete (deliberate, like SQL) | Unbounded reasoning loops |

Engram is **complementary** to LLMs, not a competitor: known paths resolve without the model;
novel cases escalate to the LLM with a structured handoff; the LLM's confirmed answers write
back into the graph as a teaching signal (future.md §19.1, use_cases §8).

### 4.4 Market and commercial potential (honest version)
The revenue thesis is narrow and CFO-legible: **reduce LLM API spend by 70–80% in bounded
domains** (use_cases §8.4). Products: the agent-mesh cost optimizer; the MCP knowledge server
(Engram as durable, confidence-weighted memory for LLM agents — distinct from RAG: typed paths
with confidence and ruled-out candidates, not text chunks, §9); the tool-security boundary
(structural impossibility of forbidden actions, §11). The moat compounds automatically: every
confirmed session strengthens the graph, and the graph is portable between deployments
(distributed merge, future.md §20.7). Market size is **not claimed** here — the §17.5 metrics
on a real domain are the validation, not forecasts.

### 4.5 Evidence — before and after the gate

| Stage | Evidence | Investor role |
|---|---|---|
| Now | Complete design docs (12 files) + a Rust CLI implementation in the repo (Apache-2.0); phase-completion status beyond the skeleton is not yet proven | **None requested.** Developer self-funds. Watch. |
| After Experiment 1 | `--explain` activation trace demo | Inbound interest via Show HN / Reddit |
| After Experiment 2 | Breaking-question decomposition demo, `paths.json` | Community engagement |
| After Experiment 3 (**gate**) | Zero-API-call resolution + the §17.5 metrics | Investors may approach — proofs are public |
| After Experiment 4 | One specialist domain at the §8.4 cost profile | Seed conversations open |
| After production MVP | Cost-optimizer deployment with measured savings; first users | Initial ask: £40k fixed, milestone-gated |

### 4.6 Answers to the standard objections

| Objection | Answer |
|---|---|
| "Not Turing-complete — too limited." | Deliberate design choice (future.md §19.2): bounded traversal guarantees termination and auditability — the same trade-off as SQL and regular expressions. Novel cases escalate to the LLM; generality is not lost, it is routed. |
| "A graph can't match LLM quality." | True for novel, unbounded, multilingual reasoning — and always will be (future.md §19.1). Engram wins where LLMs lose: repetitive bounded domains, zero cost, determinism, audit. The two are complementary, not substitutes. |
| "LangChain/AutoGen already do this." | They orchestrate LLM calls; Engram resolves known patterns deterministically before any model call (future.md §19.3). Composition: orchestration frameworks call Engram as an MCP tool. Neither replaces the other. |
| "LLM providers will absorb this." | The defensible layer is the accumulated graph (switching cost compounds), the reasoning trace, and the structural policy boundary — none of which a provider can copy from a prompt. |
| "Graph quality depends on seed data and sessions." | True; that is why the first domain is narrow (CI/CD triage or log patterns) and why the primary KPI (questions per resolution, trending down) is designed to expose non-compounding learning early (metrics.md §17.5). |
| "Deterministic = fragile when the world changes." | Handled by design: edge-weight decay, exploration noise, weak memory, stale-path detection, bias audit (roadmap Phase 12), and versioned graph files. |
| "Solo developer — key-person risk." | Mitigated by the fully documented architecture (12 design docs — any capable engineer can continue), auditable traces, and staged tranches. Related projects in the same portfolio (Guild, Weave) are complementary components; raising is sequenced one project at a time. |
| "Why invest now vs. later?" | After the gate, evidence is priced in. The post-validation ask is small, tranched, and verifiable by the investor directly — the reasoning traces are the audit trail. |

### 4.7 Is this worth investing in? (the straight answer)

**After the gate: conditionally yes — as a small, milestone-gated pre-seed of £40k, not a
classic seed.** The problem is CFO-legible (LLM spend), the differentiation is structural
(determinism, audit, zero-cost known paths), and the moat compounds automatically (the graph).
The gate is mechanical: a bounded-domain query resolved entirely by the graph, zero API calls,
with the §17.5 metrics. The money only ever follows measured evidence, and the investor can
verify every claim from the reasoning traces.

**Before the gate: no money is requested at all.** The developer's position is that a
pre-proof project should not take investor money — experiments and proofs come first.

---

## 5. Where to post to attract investors

### Phase A — public proof during experiments (self-funded, costs nothing but time)

Post each passing experiment as public evidence *before* any investor conversation.

- **Hacker News — Show HN:** after Experiment 1: "Show HN: Engram — a deterministic reasoning
  kernel; 100 MB, no GPU, resolves a bounded domain in microseconds with a full reasoning
  trace." After the gate: the zero-API-call trace + the metrics table is the money shot.
- **Reddit:** r/LocalLLaMA, r/MachineLearning, r/rust, r/ExperiencedDevs — short posts with the
  demo and the mechanical pass criteria. r/LocalLLaMA rewards token-economy stories.
- **X/Twitter:** AI-builders community; demo clips of `--explain` and the cost-profile numbers;
  threads, not manifestos.
- **Written piece:** "Stop paying for the same reasoning twice" — the recurring-problem /
  zero-API-cost argument on a personal blog, dev.to, or Medium; link the repo and reports.

### Phase B — investor platforms (only after the gate)

- **Y Combinator — Startup School + batch application.** Solo-founder friendly; the staged,
  rate-derived ask fits "small team, high leverage".
- **Wellfound (AngelList)** — profile + pre-seed listing; target angels who fund open-source /
  AI infrastructure / dev tools.
- **F6S** — pre-seed listings. **Investor Hunt** — active angels/VCs by focus.
- **Vestbee / SeedBlink / EU-angel networks** — EU-focused angels; many EU countries offer
  angel-investor tax incentives.
- **Dealroom / Crunchbase** — list the project for inbound visibility once demos exist.

### Phase C — non-dilutive and grants (can run in parallel with the experiments)

- **NLnet Foundation** — EU grants for open-source internet and AI infrastructure (fit: small,
  auditable, privacy-preserving, offline-capable).
- **European Innovation Council (EIC) / Horizon Europe** small grants; national innovation
  programs.
- **GitHub Accelerator / GitHub Sponsors, OpenCollective, Polar.sh** — open-source funding that
  also builds a contributor community.
- **Mozilla Builders** and similar AI-focused programs when open.

### Sequencing

1. Run experiments 1–4, self-funded; post each passing gate publicly.
2. Parallel: apply for non-dilutive grants (NLnet, GitHub Accelerator, national programs).
3. Only after the zero-API-call gate and public proofs: list on Wellfound/F6S with the metrics
   report as evidence; approach 5–10 angels with a concrete ask — £40k total (2 × £20k
   tranches), milestone-gated.
4. Two or three checks are enough to start; the gate structure lets later investors join at
   the next milestone with more evidence.

---

## 6. Document map

| Topic | Where |
|---|---|
| Proposal hub (objective, core concept, use cases) | `docs/proposal.md` |
| Architecture, data structures, policy engine | `docs/architecture.md` |
| Breaking questions, path labeling, goal tracking | `docs/disambiguation.md` |
| Learning, reinforcement, latent nodes, weak memory | `docs/learning.md` |
| Storage backends, memory layout | `docs/storage.md` |
| Knowledge base, context expansion, noise, bias | `docs/knowledge.md` |
| System size and outcome metrics | `docs/metrics.md` |
| Development phases 0–15 | `docs/roadmap.md` |
| Comparison, future directions, summary | `docs/future.md` |
| Deployment contexts and strategic priority | `docs/use_cases.md` |
| Master navigation index | `docs/proposal-index.md` |
| This memo | `docs/investor.md` |
