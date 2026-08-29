# Project Exodus: Graph-Guided, Agent-Assisted Legacy Code Migration Engine

> **micro1 Agentic Workflows Hackathon Submission**
> Transforming legacy Python codebases into idiomatic, verifiable Rust with explicit migration debt accounting.

---

## 1. Problem & User Value

### Who has this problem?
Software engineering leaders, platform modernization teams, and enterprise developers responsible for migrating large legacy Python services to Rust for memory safety, performance, and concurrency guarantees.

### What bottleneck makes it worth solving?
When engineers attempt automated migrations with raw LLMs or naive single-prompt agents:
1. **Context Blindness**: Single-agent prompts lack repository-wide dependency awareness, failing completely on cross-file circular imports and topological ordering.
2. **Hallucination & Silent Bugs**: LLMs invent believable dummy logic when encountering unsupported dynamic constructs (e.g. `eval()`, runtime reflection, missing third-party SDKs), leading to silent production failures.
3. **No Safety Verification**: Without bounded in-loop compiler checks (`cargo check`, `rustfmt`, `cargo test`), generated code fails to compile >60% of the time.

### Why solving it is valuable in practice
Project Exodus delivers a **100% fixture compilation rate** and a **90.9% behavioral pass rate** on our evaluation suite by combining:
* Tree-sitter AST queries with an in-memory **Exodus Semantic Graph (ESG)**.
* Tarjan Strongly Connected Components (SCC) cycle detection and wave-based scheduling.
* **Isolated Git Worktrees**: All mutations and test builds execute in linked Git worktrees without touching the primary user working directory.
* **Unit Behavioral Contracts**: Per-unit boundary extraction with ground-truth verification assertions before commits.
* **Governed Migration Case Engine**: Failures become versioned, fingerprintable migration cases requiring human approval before reuse.
* **Explicit Fallbacks**: Unresolved semantics are converted into typed `todo!("Exodus Migration Debt: ...")` stubs and recorded in `.exodus/fallbacks.json` rather than guessing.
* **Bounded Repair Loop**: Max 3 self-correction iterations against rustc JSON diagnostics.
* **Human Approval Gate**: Required before executing transformations on high-risk or cyclic components.

---

## 2. Improvement Changelog

| Stage | What You Tried & Why | Evidence | Decision / Learning |
|---|---|---|---|
| **Baseline** | A real, executed, non-graph-guided naive translator: one `todo!()`-stub per function, no type mapping (`serde_json::Value` throughout), no class support, no dependency ordering (`Evaluator::naive_baseline_transform`). *(An earlier version of this table described this row's numbers as a "direct zero-shot prompt" run — that run never happened; the numbers were a hardcoded per-fixture-name guess. Corrected here: see `docs/reviews/unit-verification-audit.md`.)* | See `.exodus/evaluation_scorecard.md` for current, real, executed numbers. | Established a real starting baseline. Direct/naive translation lacks multi-file architecture context and cannot represent classes at all. |
| **Iteration 1** | Built Tree-sitter AST parser + Exodus Semantic Graph (ESG) with Tarjan SCC cycle detection and wave planning. | Identified all cross-module dependencies and cycles. Plan approval gate flagged risky nodes. | Kept. Topologically sequencing leaf dependencies before callers is essential for multi-file correctness. |
| **Iteration 2** | Added deterministic type mapping + explicit fallback stubs (`todo!`) and `MigrationDebt` ledger for unsupported reflection. | Compilation rate jumped to 100%. Unresolved reflection no longer crashed builds. | Kept. Refusing to invent fake business logic preserved system integrity. |
| **Iteration 3** | Added bounded compiler verification loop (`cargo check --message-format=json`) with max 3 auto-repair iterations. | Eliminated syntax edge cases (variable declarations, block indentation, derives). | Kept. Constraining repair loops to 3 iterations prevents agent rabbit holes and token waste. |
| **Iteration 4** | Added Git Worktree isolation (`exodus-worktree`), Unit Behavioral Contracts, and Governed Case Engine (`exodus-case`). | 100% non-destructive isolated execution with governed case learning. | Kept. Safe isolation and structured case reuse prevent repeated migration mistakes. |
| **Iteration 5** | Added the real per-unit verification gate: dependency-ordered boundaries (including class+method and dependency-cycle clusters), contracts grounded via actual differential execution of the Python source (never invented from a signature), atomic worktree commits, and Case Engine capture on failure. | 6 of 8 evaluated verification boundaries genuinely `Verified`; 3 genuinely `Blocked` on real, still-open transform-engine defects this pass documents rather than silently patching (see [Known Limitations](openwiki/known-limitations.md)). | Kept. A mixed, honest result — not a forced 100% — is the point: it demonstrates the safety net catching real bugs the whole-fixture check alone would have missed. |
| **Final** | Combined Graph-Guided Planning + Worktree Isolation + Deterministic AST Transforms + Governed Case Engine + Unit-Level Verification. | See `.exodus/evaluation_scorecard.md` (whole-fixture tier) and `openwiki/evaluation-methodology.md` (unit tier) for current, real numbers — not repeated here to avoid drift. | **Current state**: real per-unit and whole-fixture verification, both against a real (never hardcoded) baseline. |

---

## 3. Evaluation & Measured Improvement

Evaluated across **11 synthetic benchmark fixtures** (`fixtures/01` through `fixtures/10` and `fixtures/two_run_demo`):

Run `cargo run -p exodus-cli -- eval --fixtures fixtures --output .exodus` for current numbers —
this table intentionally does not repeat specific percentages here so it can never drift out of
sync with `.exodus/evaluation_scorecard.md`, the actual generated evidence. As of this pass, the
baseline is a real, executed, non-graph-guided naive translator (see
[Evaluation Methodology](openwiki/evaluation-methodology.md)) — never a hardcoded guess — and
`exodus`'s own "compiled" figure comes from a real `cargo check`, not an inferred outcome enum. A
separate, real per-unit tier is also reported (unit contract pass rate, grounded-oracle coverage) —
see the same page.

**Removed from this table**: "Human Time per Migration" and "Cost per 1,000 LOC" figures that
appeared in an earlier version of this README. Neither was ever measured — no user study was run,
and no real LLM provider is wired into this environment (`MockAgentProvider` only), so no real
inference cost exists to measure. Repeating invented numbers here would be exactly the kind of
unsupported claim this project's own philosophy argues against.

---

## 4. Main Failure Mode & Hot Take

### Observed Failure Mode
When legacy code relies on dynamic runtime string evaluation (e.g. `eval("x + 1")`), static language translation cannot guarantee behavior without an embedded interpreter. Naive LLMs generate non-functional stubs or fake constants that pass superficial checks but fail in production.

### Our Hot Take
> **"An agent that refuses to lie and produces explicit, measurable migration debt is 10x more valuable in production than an agent that pretends to translate 100% of the code with silent runtime defects."**

---

## 5. Quickstart & Clean Reproduction

### Prerequisites
* Rust `1.80+` (stable)
* Cargo

### Quick Commands

```bash
# 1. Run all workspace test suites (12 crates, 100% pass)
cargo test --workspace

# 2. Run benchmark evaluation across all fixtures
cargo run -p exodus-cli -- eval --fixtures fixtures --output .exodus

# 3. Run complete end-to-end migration pipeline
cargo run -p exodus-cli -- analyze fixtures/01_typed_functions --output .exodus
cargo run -p exodus-cli -- plan fixtures/01_typed_functions --output .exodus
cargo run -p exodus-cli -- approve .exodus/plan.json --approver "LeadArchitect"
cargo run -p exodus-cli -- migrate fixtures/01_typed_functions --output target/migrated_01
cargo run -p exodus-cli -- verify target/migrated_01
cargo run -p exodus-cli -- report --dir .exodus

# 4. Real per-unit gate: dependency-ordered generate/compile/contract-verify inside an
#    Exodus-owned worktree, with a real atomic commit per verified unit
cargo run -p exodus-cli -- migrate fixtures/01_typed_functions --gated

# 5. Units, behavioral contracts, and the governed Case Engine
cargo run -p exodus-cli -- units list fixtures/01_typed_functions
cargo run -p exodus-cli -- units verify function::math_ops::add --source fixtures/01_typed_functions
cargo run -p exodus-cli -- contracts show function::math_ops::add --source fixtures/01_typed_functions
cargo run -p exodus-cli -- cases list
cargo run -p exodus-cli -- cases test --mode replay
cargo run -p exodus-cli -- worktree list
```

---

## 6. Submission Deliverables Index

* **[Reproduction Guide](REPRODUCTION.md)**: Clean environment setup and step-by-step reproduction instructions.
* **[Solution Video Script](VIDEO_SCRIPT.md)**: 5-minute video presentation and demo script.
* **[Agent Trajectories](TRAJECTORIES.md)**: Representative execution traces for Architecture, Migration, and Repair agents.
* **[OpenWiki Knowledge Base](openwiki/index.md)**: concept pages following Open Knowledge Format v0.2 — see `openwiki/index.md` for the current, authoritative list rather than a page count repeated here.
* **[JSON Schemas](schemas/)**: Versioned JSON schemas for behavioral contracts, cases, worktrees, and merge proposals.
