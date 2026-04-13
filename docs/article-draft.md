# Article Draft — Engram: The Missing Layer Between Your LLM and Deterministic Knowledge

**Status:** Working draft — not for publication yet  
**Audience:** Engineers, AI practitioners, LLM tooling community  
**Timing context:** The LLM Wiki pattern (Karpathy, April 2026) is trending — this positions Engram as the next logical step in the same conversation.

---

## Working Notes (remove before publication)

- Tone: technical but accessible, honest about early stage, no hype
- Do NOT overstate what Engram can do today (Phase 1 — keyword lookup with reasoning trace)
- DO state clearly what the architecture is designed to become
- The article is a design proposal going public, not a product launch

---

## Title Options (pick one)

1. Engram: The Missing Layer Between Your LLM and Deterministic Knowledge
2. What If Your LLM Never Had to Answer the Same Question Twice?
3. Beyond RAG and LLM Wikis: Compiling Knowledge into Executable Reasoning

---

## 1) The Problem

Large language models are remarkable — and enormous. Typical models carry tens
of gigabytes of weights and often require substantial RAM/VRAM at runtime.
For practical throughput they typically need a GPU or a remote API call, and
they still re-reason from scratch on every query.

In production terms, the pain is clear:

- expensive at runtime
- non-deterministic by design
- difficult to audit step-by-step
- forgetful across sessions unless memory is engineered externally

For many real-world workloads, this is unnecessary overhead.

## 2) The Observation

Most operational questions are not open-ended intelligence tasks. They are
repetitive, structured, and bounded: the same error signatures, the same
workflow branches, the same resolution paths.

A system that has seen a question resolved correctly once does not need
billions of parameters to rediscover the same reasoning forever.

## 3) The Insight

We often do not need fresh intelligence at query time.
We need compression of repeated reasoning.

In other words: capture confirmed resolution paths once, then replay them
deterministically at near-zero marginal cost.

## 4) The Idea

Treat knowledge as evolving graph structure, not ephemeral conversation state.
Nodes encode concepts and decision points; edges encode confidence-weighted
transitions learned from confirmed outcomes.

The graph does not memorize raw text. It accumulates reusable reasoning paths.
Each confirmed resolution strengthens a route; rejected paths decay.

## 5) The System

That is Engram: a deterministic reasoning layer that sits in front of an LLM.
It resolves well-trodden paths directly and only escalates unresolved cases
after a bounded graph loop.

The first version was narrow: a standalone reasoning engine for bounded
domains. Useful, but limited. After a few iterations, two things became clear:

**First, the system sits naturally in front of an LLM — not instead of one.**
Engram handles the well-trodden 80–95% of queries in a bounded domain without
any model call. When confidence is below threshold, it first runs a bounded
iteration loop inside the graph: proactive fetch actions (if configured), then
breaking questions (if still ambiguous), each cycle re-entering activation with
the accumulated context. Only if no confident path emerges within those bounds
does it hand off to the LLM with structured context: the graph path traversed
so far, the candidates ruled out, the confidence state. Not raw conversation —
a typed reasoning payload. The LLM resolves the novel case, and that resolution
writes back into the graph as a new path. That query never costs tokens again.

```text
Query arrives
  │
  ▼
Engram graph activation
  │
  ├── High confidence ──► Answer directly   (no API call, microseconds)
  │
  └── Confidence below threshold ──► Bounded graph loop
          │
          ├── Fetch actions + breaking questions + re-propagation
          │
          ├── If confidence rises ──► Answer directly
          │
          └── If still unresolved ──► Structured handoff to LLM
                    │
                    ▼
              LLM response → reinforces the graph for next time
```

The crossover point is coverage: as the graph accumulates session data,
the proportion of queries requiring the LLM drops. In a well-covered
domain, 70–90% of queries resolve from the graph alone. The LLM becomes a
teaching signal, not a runtime dependency.

**Second, exposed via MCP, the graph becomes the LLM's persistent memory.**
Today's LLMs carry memory in context windows (resets every conversation) or
flat files (static, unstructured). Engram exposed as an MCP tool gives any
LLM agent access to persistent, confidence-weighted, self-improving knowledge
accumulated across thousands of sessions. The LLM calls `engram.query()`
mid-reasoning and receives a typed reasoning path — not a text chunk.
Multiple agents sharing one Engram instance share the graph (compressed
patterns), never raw conversations.

This was not the original design goal. It emerged from the architecture.

## 6) The Positioning

This is not Engram versus LLMs. It is division of labor.

- LLM = exploration
- Engram = exploitation

The LLM explores novel space and teaches new paths. Engram exploits what is
already known with deterministic, auditable, low-cost execution.

---

## How a Query Actually Moves Through the Graph

The processing pipeline is a loop, not a straight line. A query may pass
through the engine multiple times — each pass enriched with additional
context — before reaching a confident answer or escalating.

**Step 1 — Tokenize.** Input text — whether from a human operator (HO) or
an LLM caller — is mapped to node IDs. The text is discarded at this
boundary — nothing downstream holds caller-originated strings. For MCP
callers, the LLM sends a pre-formed array of concept identifiers, so this
step becomes a trivial lookup.

**Step 2 — Activate.** Matched nodes receive initial activation scores
based on match quality.

**Step 3 — Propagate.** Activation spreads forward through outgoing edges,
attenuated by edge weight and a decay factor. Multi-hop propagation
allows the signal to reach nodes that no input token matched directly —
the graph infers context that the LLM/HO did not state.

**Step 4 — Rank candidates.** Solution nodes are ranked by accumulated
activation. The engine evaluates confidence: is the top candidate strong
enough, and is the gap to the second candidate wide enough?

**Step 5 — Decide.** Two outcomes, determined by confidence state:

```text
Activation + propagation
  │
  ├── High confidence ──► Return answer directly
  │
  └── Confidence below threshold ──► Bounded iteration loop:
        │
        │  Each cycle re-enters at Step 2 (Activate) with:
        │    = original query node activations      (always re-applied)
        │    + fetch action results                 (see note below)
        │    + breaking question answers            (tokenised to node IDs)
        │
        │  The union of those sets is the next activation seed.
        │  The graph propagates forward from there.
        │
        │  Note — fetch action result mapping:
        │    Each action definition declares expected response states
        │    and their corresponding node activations directly:
        │      RunHealthCheck → {ok: [], degraded: [node:db_slow],
        │                        down: [node:db_unreachable]}
        │    For richer responses (logs, JSON), the response text is
        │    tokenised through the same pipeline as input — a hybrid:
        │    structured mapping for known states, tokeniser for the rest.
        │
        ├── [if fetch actions configured]
        │     Run proactive context sweep
        │     → results activate additional nodes
        │     → re-propagate  ──► loop
        │
        ├── [if still ambiguous]
        │     Ask breaking question
        │     → LLM/HO response activates additional nodes
        │     → re-propagate  ──► loop
        │
        └── Exit — handoff to LLM/HO with full session context (any of):
              - no candidates activated (graph has no path for this input)
              - all reachable paths tried and exhausted
              - recursion depth limit reached
```

The fundamental shape of each re-entry is:

```text
original query
  → node activations A

fetch results + breaking question answers
  → node activations B

A ∪ B  ──► Step 2 (Activate) ──► Step 3 (Propagate) ──► Step 4 (Rank)
```

The original query activations `A` are always re-applied — the same initial
nodes fire every cycle. Each cycle only adds to the activation set; it never
discards what came before. If additional context `B` pushes the graph toward
a different path, confidence rises and the loop exits with an answer. If `B`
only reinforces the same path — because the graph genuinely has no other route
— confidence stays below threshold and the loop exhausts its budget before
handing off. That edge case is acceptable: the same path re-confirmed with
richer context is still a signal worth passing to the LLM/HO.

**How fetch action results become node activations** is a deliberate design
point. Two approaches work together:

- **Structured mapping** — the action definition declares a fixed set of
  response states and the node activations each one triggers:
  `RunHealthCheck → {ok: [], degraded: [db_slow], down: [db_unreachable]}`.
  This is fully auditable — every possible activation from every action is
  enumerable at deploy time. The nodes involved should already exist in the
  graph as meaningful concept anchors; the action mapper routes to them,
  it does not create new ones.

- **Tokeniser fallback** — for richer responses (log snippets, JSON payloads,
  status strings), the response text passes through the same tokeniser pipeline
  as input text, mapping tokens to node IDs. Less precise, but handles novel
  or variable responses gracefully.

Both paths converge to node IDs before re-entering the activation step. The
structured mapping handles typed, expected states deterministically; the
tokeniser handles the rest. Using both avoids both over-rigidity (only
structured) and over-looseness (only tokeniser).

The loop exits on any of three conditions: no candidates activated at all
(the graph has no path for this input), all reachable paths tried and
exhausted, or the configured recursion depth reached. Engram can be a large
graph spanning thousands of nodes — the depth limit is what keeps any single
session bounded, regardless of graph size.

On exit, the session hands off to the LLM/HO with the complete accumulated
context: every node activated, every fetch result, every breaking question
answer gathered across all iterations. The LLM/HO is responsible for
resolving the case from that point. If the LLM/HO reaches a confirmed answer,
that resolution writes back into the graph as a new path — so the same query
costs nothing the next time.

The LLM/HO is the last resort, not the first. Fetch actions and breaking
questions run first — the LLM/HO only sees cases the graph genuinely could
not resolve within its configured bounds.

---

## Where the LLM Wiki Pattern Fits — and Where It Stops

In April 2026, Andrej Karpathy proposed what is becoming known as the LLM
Wiki pattern: instead of using RAG to retrieve raw documents at query time,
use an LLM to compile knowledge into a maintained Markdown vault at ingest
time. The knowledge gets compiled once, then stays compiled. The LLM
maintains cross-references, flags contradictions, keeps summaries current.

The core shift — from retrieval to compilation — is exactly right. RAG
rediscovers everything from scratch on every query. The wiki pattern compiles
once and queries against the compiled state. That is a genuine architectural
improvement.

But Markdown is a human-readable format, not a machine-executable one. When
you query an LLM Wiki, the LLM still has to read the wiki pages, interpret
them, and synthesize an answer. The compilation step is real — but the query
step is still stochastic. Ask the same question twice and you are not guaranteed the
same answer. The LLM is still in the critical path at query time.

The determinism gap matters more than people assume — for two reasons.

**Consistent answers reduce cognitive load in operational contexts.** When
operating a system or following a runbook, people apply existing knowledge
faster when the same query triggers the same classification, the same steps,
the same terminology. When an LLM rephrases or restructures its answer on
every query — even if both answers are correct — the operator has to re-parse,
re-map, and re-verify rather than trusting what they recognise. For procedural
reference work, consistency compounds reliability. Stochastic variation
creates unnecessary overhead.

**Reliable tooling requires deterministic foundations.** Today we struggle to
build tools that depend on LLM outputs because those outputs shift between
calls. A monitoring dashboard, a routing decision, an automated triage
pipeline — each of these needs to trust that the same input produces the same
classification, the same confidence score, the same action. Composing
reliable systems on top of non-deterministic components is significantly
harder — it requires additional abstraction layers such as retries, consensus
mechanisms, and output validation, each adding cost and complexity. Determinism
is not just a nice-to-have — it is a prerequisite for the next layer of tooling
around LLMs.

Engram takes the compilation idea further: instead of compiling into prose,
compile into a weighted directed graph where the compilation *is* the
reasoning. A query doesn't require interpretation — it activates nodes,
propagates through weighted edges, and produces a deterministic path. Same
input + same graph state, same output, every time. No model in the loop for known paths.

| Dimension | RAG | LLM Wiki | Engram |
| --- | --- | --- | --- |
| When knowledge is processed | At query time | At ingest time | At session-confirmation time |
| Query-time compute | LLM re-derives every answer | LLM reads compiled pages | Graph traversal — no LLM needed for known paths |
| Determinism | No | No | Yes — same input, same path |
| Cross-references | Discovered ad hoc | Maintained by LLM | Structural — edges in a graph |
| Privacy | Raw text stored | Raw text in wiki pages | Input text discarded at tokeniser boundary |
| Output | Temporary chat response | Persistent Markdown | Typed action contract with confidence score |

This is not an argument against the LLM Wiki pattern. It is an argument that
the two are complementary. The wiki accumulates human-readable understanding.
Engram accumulates machine-executable reasoning paths. A domain expert could
maintain both: the wiki as the inspectable narrative layer, the graph as the
deterministic query engine. As the graph matures, the LLM that maintains the
wiki increasingly focuses on genuinely novel edges — the two systems feed
each other.

---

## The Privacy and Security Argument

This one was not designed in — it fell out of the architecture.

Input text — from any caller, LLM or human operator — is discarded at the
tokeniser boundary. It never enters any storage layer. What the graph stores is a node activation pattern and an
outcome — not words, not who was involved, not what was typed. After 30
engineers hit the same error and confirm the same fix:

```text
error=timeout + service=auth
  → CheckConnectionPool  [weight: 0.91, n=34]
```

The 34 people who contributed that weight are structurally absent — not
scrubbed, never recorded. This is not a privacy policy. It is a structural
property of the data representation. There is no raw content to leak because
raw content never exists in storable form.

For LLM deployments specifically, Engram sitting in front of the model acts
as a **structural security boundary**. Current LLM tool security relies on
system prompts and runtime checks — mechanisms that a sufficiently adversarial
input can circumvent. When an LLM calls Engram via MCP, the only operations
available are those explicitly enumerated in the action contract. The LLM
cannot trigger an action outside the contract — not because a prompt says so,
but because the execution pathway does not exist. The action surface is
enumerable before deployment. Every blocked call is logged.

This is guardrails by architecture, not by instruction.

---

## From Documents to Knowledge Artifacts

Today, industry knowledge lives in documents. Standards, runbooks, compliance
rules, business logic — all encoded as prose that humans read and interpret.
When an LLM needs to reason about an ISO standard or a company's escalation
policy, it reads the document and hopes to extract the right rule. The
document is passive. The interpretation is stochastic.

What if the standard itself were an interactive, queryable artifact? Not a
PDF. Not a wiki page. A weighted graph that any LLM or human operator can
talk to directly — activating nodes, receiving deterministic answers,
following auditable paths. The knowledge is not described; it is wired.

This is what Engram becomes when you load a domain into it. A compliance
standard wired into an Engram graph is not a reference document — it is a
running process. An LLM queries it via MCP and gets a typed reasoning path
with a confidence score, not a text excerpt to interpret. A human operator
queries the same graph through a CLI and gets the same answer. The artifact
is the interface — for both.

**The brain analogy.** If you model a working human mind, the LLMs are the
processing functions — context creation, abstraction, generalisation,
synthesis, sensory processing, environment interaction. Engram is the wired
storage beneath them: the accumulated operations, customs, behaviours, ideas.
A persona is not a model — it is a set of wired responses shaped by
experience. Every interaction with a human, every confirmed resolution, every
corrected mistake — that is knowledge extraction, and the graph is where it
crystallises.

This is what an engram is in neuroscience: the physical trace left by an
experience. Not the experience itself — the structural change it leaves behind.
The name is literal.

**LLM Machine Interface.** Industry already has HMI — Human-Machine
Interface — as a foundational concept. Engram enables the next layer: **LMI —
LLM Machine Interface**. Where HMI maps human intentions to machine
operations through buttons, screens, and control panels, LMI maps LLM
reasoning to machine operations through a deterministic knowledge graph.

```text
HMI:  Human   ──► buttons / screens / controls  ──► Machine
LMI:  LLM/HO  ──► Engram knowledge graph        ──► Machine / Process
```

The knowledge graph *is* the interface. It defines what operations exist,
what paths lead to them, what confidence is required, and what policy gates
apply. The LLM does not call arbitrary functions — it navigates a graph that
was designed, audited, and approved before deployment. The human operator
navigates the same graph. Both get deterministic, auditable outcomes.

This positions Engram not as a novel invention, but as a deliberate step
back to something the industry already uses daily wherever determinism in
operations matters. State machines drive embedded controllers, protocol
implementations, and UI frameworks. PLC units wire deterministic logic
electrically into manufacturing lines, power plants, and safety systems —
no ambiguity, no stochastic interpretation, every state transition traceable
and, in safety-certified deployments, fully auditable.
These approaches are foundational precisely because they are bounded,
inspectable, and provably correct.

Engram applies the same principle to knowledge and reasoning: **concrete,
stable, inspectable operations** — now extended to work natively with both
LLMs and humans as first-class callers. The novelty is not the determinism.
The novelty is making it the interface between an LLM and the world.

---

## The Industry Bridge

There is a gap in the current AI landscape that keeps widening: LLMs are
increasingly capable, but industries that require determinism — medical
triage routing, financial compliance, infrastructure fault isolation,
safety-critical systems — cannot rely on them as sole reasoning engines
without extensive guardrails. The outputs are stochastic. The reasoning
is opaque. Auditability is approximate at best.

Engram is formally a finite state machine with weighted transitions and an
online learning mechanism — strictly weaker than a Turing machine, and that
is the point. Turing completeness and the Halting Problem are inseparable: no general
algorithm can decide whether an arbitrary Turing-complete program will
terminate. Engram guarantees termination — every session resolves or
escalates within bounded steps. For regulated environments, that guarantee matters more than
generality.

The same system can be configured and extended by both AI assistants and
human operators. The graph files are plain JSON — a human can inspect, edit,
or approve every node and edge. An LLM agent can propose new paths from
escalation outcomes. A compliance officer can freeze the graph for audit.
A domain expert can load a new knowledge file without touching the engine.

This configurability is what connects the LLM world to the industrial world:
the LLM teaches the graph in development; the graph runs deterministically
in production. The foundation for an **industrial brain**: Engram as the
wired knowledge layer, LLMs as the adaptive reasoning layer, human operators
as the curators and auditors.

---

## What Engram Is Today — Honestly

Phase 1 of 15 is complete. The Rust binary compiles, loads a real knowledge
graph (19 nodes, 17 edges — HTTP/API error domain), and answers queries by
keyword lookup with an optional reasoning trace. The subcommands for weak
memory, latent node discovery, provisional nodes, and bias audit exist as
stubs — each maps to a future phase.

What exists is the architecture, the design documentation, and a working
skeleton. What does not exist yet is activation propagation, breaking
questions, session recording, reinforcement learning, or MCP integration.
The roadmap has 15 phases. The engine is built to be filled in, not
redesigned.

The design is published as open-source (Apache 2.0). The knowledge file
format (JSON) and the reasoning specification are language-agnostic —
implementations in any language are welcome.

---

## The Bet

The bet is simple: in bounded domains, the same 200–2000 problem signatures
account for 80–95% of real-world queries. A graph encoding those signatures
resolves them in microseconds. Every LLM resolution of a novel case becomes
a new graph path — compounding the coverage automatically.

LLMs will keep getting better at novel reasoning. Engram is not competing
with that. The bet is that there will always be a large class of queries
where deterministic, auditable, instant, offline resolution is worth more
than generality — and that class is underserved today because the tooling
assumes every query needs a model.

---

## Open Questions and Known Gaps

This article describes a design proposal, not a shipped product.
Engram emerged from several months of iterative exploration and refinement, culminating in a focused design phase over the past month. The current implementation is in its early stages, with an estimated six months of development to reach production-level capability.

**New terminology.** Several terms in this article — **LMI** (LLM Machine
Interface), **LLM/HO** (LLM or human operator as interchangeable callers) —
are coined here to describe patterns we observe, not established industry
vocabulary. If you search for "LMI" today, you will not find a standards body
or a Wikipedia page. We believe these patterns need names because they
describe real architectural roles that current terminology does not cover.
But the reader should know we are proposing these terms, not citing them.

**Graph authoring is a hard upstream problem.** The article describes loading
a compliance standard or a business domain into an Engram graph as if it were
natural. It is not. An ISO standard is hundreds of pages. Converting it
faithfully into a weighted graph without losing edge cases is a significant
effort — arguably comparable to writing traditional code for the same domain.
The Engram engine handles the reasoning; it does not solve the knowledge
engineering problem of who creates the graph and how they verify its
completeness. Initially, graphs are maintained as JSON configuration files —
hand-authored or generated by tooling. Later roadmap phases plan a visual
graph editor for adjusting connections and weights, and a dedicated
configuration language or Markdown-based format for describing graph
structure more naturally. LLM-assisted graph authoring from existing
documentation is a plausible longer-term path. But today, graph authoring
is manual and the tooling story is not yet written.

**The persona claim is a simplification.** The article draws an analogy
between Engram and the wired substrate of a human persona — the stored
patterns that define how someone responds. This captures something real:
Engram stores *operational persona* — which paths to prefer, which actions to
take, which resolutions to trust. But a full human persona also includes
emotional tendencies, aesthetic preferences, communication style, values —
all things that live in the stochastic LLM layer, not in a deterministic
graph. The claim that "a persona is a set of wired responses shaped by
experience" is the author's personal simplification of a genuinely complex
problem in neuroscience and cognitive science. It is useful as a framing
device; it is not a scientific claim.

**The tokeniser fallback for action responses is fragile.** The article
describes two approaches for mapping fetch action results to node activations:
structured mapping (reliable) and tokeniser fallback (pragmatic). The
tokeniser fallback — running arbitrary JSON or log output through keyword
extraction and hoping meaningful node IDs fall out — is genuinely fragile.
A log line like `ERROR pool exhausted: max_connections=50` would need
`pool` and `exhausted` to match graph node labels. This works in narrow
domains with controlled vocabularies; it does not generalise. The structured
mapping is the reliable path. The tokeniser fallback is an escape hatch with
known limitations, not a solved design.

**The 80–95% and 70–90% figures are projections.** These numbers describe
what the architecture aims to achieve in well-covered bounded domains. They
are informed estimates based on the distribution of recurring queries in IT
operations, customer support, and CI/CD triage — domains where the pattern
is well-documented. They are not measured results from Engram deployments.
The system is in Phase 1 of 15; production measurements do not yet exist.

**The neuroscience analogy is an analogy.** The term "engram" in neuroscience
refers to the physical changes in brain state (synaptic plasticity, protein
synthesis, reconsolidation) that constitute a memory trace. Engram the system
is *inspired by* this concept — a structural change left by a confirmed
experience — but it is not a neuroscience implementation. The weight update
rule is Rescorla-Wagner (a learning model from behavioural psychology), not
a biophysical simulation. The name is metaphorical, chosen deliberately, but
the reader should not take it as a claim of biological equivalence.

---

## Links

- **Repository:** [github.com/dominikj111/Engram](https://github.com/dominikj111/Engram)
- **Design proposal:** [docs/proposal.md](https://github.com/dominikj111/Engram/blob/main/docs/proposal.md)
- **Roadmap (15 phases):** [docs/roadmap.md](https://github.com/dominikj111/Engram/blob/main/docs/roadmap.md)
- **Use cases (10 deployment contexts):** [docs/use_cases.md](https://github.com/dominikj111/Engram/blob/main/docs/use_cases.md)
- **Contributing:** [CONTRIBUTING.md](https://github.com/dominikj111/Engram/blob/main/CONTRIBUTING.md)

---

Best slot for posting: Thursday, 8–9am UK time (GMT+1 in April)

What that covers simultaneously:

UK/EU: morning commute and first-coffee reading window (8–10am)
US East Coast: 3–4am → will surface in feeds by the time they wake up (7–9am ET), so it already has early engagement signals by the time the largest US tech audience opens their apps
US West Coast: midnight → algorithm has 5–6 hours to index and recirculate before their morning
The EU audience is your easiest win from that slot — Germany, Netherlands, and the Nordics (dense engineering communities) are fully active at 8am UK.

One practical tip: Pin the LinkedIn post and X/Twitter post to your profile and engage actively in the first 2 hours after posting. Early comments and replies are the strongest engagement signal both platforms use to decide whether to push the content further. Even replying to your own post with a one-line follow-up thought resets the recirculation clock.

So: next Thursday, 8am UK — if the article is ready.

## Fact-Check Notes (remove before publication)

All claims verified against the repository documentation:

1. ✅ "80–95% of queries" — sourced from future.md §19.1 ("200–2000 problem
   signatures account for 80–95% of all queries")
2. ✅ "<100 MB memory" — sourced from proposal.md constraints
3. ✅ "microseconds on a single CPU core" — sourced from future.md §19.1
4. ✅ "70–90% of queries resolve from graph" — sourced from future.md §19.1
   ("Engram handles 70–90% of queries without any model call")
5. ✅ "finite state machine with weighted transitions" — sourced from
   future.md §19.2 formal characterisation
6. ✅ "input text discarded at tokeniser boundary" — sourced from README and
   architecture (structural privacy guarantee)
7. ✅ "Phase 1 of 15 complete" — sourced from README Status section
8. ✅ "19 nodes, 17 edges" — sourced from README Quick Start
9. ⚠️ "reduce LLM API spend by 70-80%" — sourced from use_cases.md strategic
   priority. This is a projection for well-covered bounded domains, not a
   measured result. The article frames it as "the value proposition" not a
   proven claim — acceptable, but ensure the framing stays honest.
10. ⚠️ The Karpathy LLM Wiki description is based on community reporting, not
    a direct Karpathy publication. Attribution says "what is becoming known as
    the LLM Wiki pattern" — appropriately hedged.
