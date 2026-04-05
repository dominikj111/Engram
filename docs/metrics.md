# Engram — System Size and Outcome Metrics

*Part of the [Engram design documentation](proposal.md).*

---

## 17. Expected System Size

| Component          | Estimated Size |
| ------------------ | -------------- |
| nodes + edges      | 5–30 MB        |
| named paths        | <2 MB          |
| solutions          | 1–5 MB         |
| weak memory        | <5 MB          |
| session history    | <3 MB          |
| **Total**          | **< 45 MB**    |

Well below the 100 MB constraint, leaving room for graph growth.

---

## 17.5 Outcome Metrics

Graph quality metrics (edge weights, confidence scores, latent node counts)
measure internal beauty. They do not measure whether the system is actually
solving problems for users. These are the metrics that matter in production.

| Metric               | Definition                                                    | Target    |
| -------------------- | ------------------------------------------------------------- | --------- |
| Resolution rate      | Sessions ending `Resolved` / total sessions                   | > 80%     |
| Time to resolution   | Turns from first query to `Resolved`                          | < 4 turns |
| Escalation rate      | Sessions ending `Escalated` / total sessions                  | < 15%     |
| Correction rate      | Weak memory entries / total sessions                          | < 10%     |
| Repeat question rate | Breaking questions asked for already-confirmed dims           | < 5%      |
| Cache hit rate       | Path cache hits / total queries (Phase 6+)                    | > 40%     |
| Action success rate  | Actions completed without policy block / actions selected     | > 95%     |
| User friction score  | Sessions with 3+ rejected candidates (proxy for frustration)  | < 8%      |

Metrics are derived from `sessions.json` and `weak_memory.json` — no
separate telemetry pipeline is needed. A `engram --metrics` command
computes them over the last N sessions (default: 100).

The escalation rate and correction rate are the two numbers that most
directly indicate whether the graph is fit for a given domain. A rising
escalation rate signals knowledge gaps; a rising correction rate signals
over-confident edges that need negative reinforcement. Both are actionable
without touching code — only knowledge files.
