---
okf_version: "0.2"
type: invariant
status: planned
sources:
  - openwiki/INSTRUCTIONS.md
  - crates/exodus-planner/src/lib.rs
---

# Human Approval Model

[Index](index.md) | [Migration Planner](migration-planner.md) | [Agent Boundaries](agent-boundaries.md)

Human oversight is a non-negotiable invariant in the Exodus migration engine.

## Checkpoints
* **Plan Approval**: Reviewing topological waves, high-risk symbols, and cycle breaking before any files are modified.
* **Fallback Acceptance**: Confirming explicit stub insertion and migration debt classification for unmapped constructs.
* **Destructive Transformations**: Requiring interactive confirmation for deletions, signature shifts, or breaking schema migrations.
