---
okf_version: "0.2"
type: architecture
status: planned
sources:
  - Cargo.toml
  - crates/exodus-core/src/lib.rs
  - crates/exodus-graph/src/lib.rs
---

# System Architecture

[Index](index.md) | [Semantic Graph](semantic-graph-specification.md) | [Crates](../Cargo.toml)

Exodus is architected as a modular Rust workspace divided into decoupled pipeline crates:

```text
crates/
├── exodus-core/       # Domain primitives, outcome types, errors
├── exodus-parser/     # Tree-sitter frontend abstractions
├── exodus-graph/      # Exodus Semantic Graph (ESG) structures and algorithms
├── exodus-planner/    # Topological wave planner and risk evaluation
├── exodus-agent/      # Bounded agent orchestration and prompt harnesses
├── exodus-transform/  # Semantic code generation and AST transformation
├── exodus-fallback/   # Fallback stub generation and debt emission
├── exodus-verifier/   # Compiler, linter, and test verification runners
├── exodus-eval/       # Outcome metrics and benchmark aggregation
└── exodus-cli/        # Main CLI entry point
```

## Data Flow
```
Legacy Code -> [exodus-parser] -> AST
AST -> [exodus-graph] -> ESG (Exodus Semantic Graph)
ESG -> [exodus-planner] -> Migration Plan (Waves)
Plan -> [Human Review Gate] -> Approved Waves
Approved Waves -> [exodus-transform] & [exodus-fallback] -> Target Code + Debt
Target Code -> [exodus-verifier] <-> [exodus-agent (Repair)] -> Verified Code
Outcomes -> [exodus-eval] -> Final Report & Evidence (.exodus/)
```
