# Agent Operational Guidelines & Contract

This repository governs the implementation and execution of **Project Exodus**. All autonomous agents, LLM assistants, and automated contributors must adhere to this contract.

## Non-Negotiable Invariants

1. **Evidence-Based Operation**: Never fabricate implementations or verification statuses. All claims must be backed by working Rust code, test suites, or execution artifacts in `.exodus/`.
2. **Outcome Classification**: Maintain strict separation between outcome tiers:
   * `Verified`: Passes compiler checks AND behavioral test contracts.
   * `Compatible`: Compiles cleanly without stubs, awaiting test suite validation.
   * `Degraded`: Relies on explicit `todo!` fallback stubs; recorded as `MigrationDebt`.
   * `Blocked`: Fails compilation or transformation after bounded repair attempts.
3. **Bounded Repair**: Agents working on automated code repair must not exceed **3 iterations** on a single symbol. If unresolved, emit an explicit fallback stub and record migration debt.
4. **Human Approval Gate**: Never perform destructive changes, circular dependency breaking, or irreversible transformations without generating an approval request in `.exodus/plan.json`.
5. **Separation of Concerns**: Keep the **Exodus Semantic Graph (ESG)** (program semantics model) strictly decoupled from the **OpenWiki Knowledge Base** (documentation model).
6. **Secrets & Privacy**: Never commit or emit credentials, private keys, or confidential source snippets into documentation or public artifacts.

## Crates Responsibility Matrix

| Crate | Primary Focus |
|---|---|
| `exodus-core` | Shared domain models, `MigrationOutcome`, `MigrationDebt`, error types |
| `exodus-parser` | Tree-sitter grammars, AST traversal, symbol extraction |
| `exodus-graph` | ESG representation, cycle detection (Tarjan/FAS), topological sorting |
| `exodus-planner` | Wave generation, risk weighting, approval checkpoints |
| `exodus-agent` | Bounded repair controller, LLM prompt orchestration |
| `exodus-transform` | Semantic construct translation to idiomatic Rust |
| `exodus-fallback` | `todo!` stub generation, migration debt emission |
| `exodus-verifier` | Formatting (`rustfmt`), compilation (`rustc`), test runners (`cargo test`) |
| `exodus-eval` | Evaluation metrics, benchmark scorecards |
| `exodus-cli` | Command-line dispatch and execution flows |
