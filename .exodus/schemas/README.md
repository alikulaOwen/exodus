# Exodus Schemas

This directory is a placeholder. The project's actual JSON Schemas live at the repository root in
[`schemas/`](../../schemas/), not here, and validate:

- `schemas/behavioral-contract.schema.json` — per-unit behavioral contracts
  (`.exodus/contracts/<unit-id>/behavioral-contract.json`).
- `schemas/verification.json` — per-unit verification results
  (`.exodus/contracts/<unit-id>/verification.json`).
- `schemas/case.schema.json` — Migration Case records (`.exodus/knowledge/cases/*.json`).
- `schemas/merge-proposal.schema.json` — worktree merge proposals
  (`.exodus/runs/<run-id>/merge-proposal.json`).

`graph.schema.json`, `plan.schema.json`, `debt.schema.json`, and `report.schema.json` (previously
listed here) do not exist anywhere in the repository — that was stale/aspirational documentation.
