# Project Exodus

> **A Graph-Guided, Agent-Assisted Legacy Code Migration Engine.**

Project Exodus is designed to systematically modernize legacy codebases (starting with Python 3.11 to Rust 2021) with verifiable behavioral parity.

## Core Principle

> *Reliable migration means maximizing verified behavior while turning unresolved semantics into explicit, measurable and reviewable migration debt.*

---

## Ten-Stage Pipeline Architecture

1. **Parse**: Ingest legacy source files via Tree-sitter AST queries.
2. **Exodus Semantic Graph (ESG)**: Construct a language-neutral semantic graph of symbols, types, and calls.
3. **Analysis**: Analyze dependencies, strongly connected components (cycles), and compute risk weights.
4. **Plan Generation**: Compute topological migration waves (leaves first).
5. **Human Approval**: Require explicit review and signoff before risky actions or schema modifications.
6. **Transform**: Translate supported constructs into idiomatically typed target code.
7. **Explicit Fallbacks**: Emit reviewable stubs (`todo!`) and migration debt records for unmapped semantics.
8. **Verification**: Format, compile, and execute behavioral equivalence test suites.
9. **Bounded Repair**: Run constrained repair loops (max 3 iterations) on compilation/test failures.
10. **Report**: Output separated classifications: `Verified`, `Compatible`, `Degraded`, and `Blocked`.

---

## Repository Structure

```text
.
├── Cargo.toml               # Workspace manifest
├── rust-toolchain.toml      # Pinned Rust toolchain (stable, 2021 edition)
├── package.json             # Pinned OpenWiki & tooling dependencies
├── deno.lock                # Pinned package manager lockfile
├── .nvmrc                   # Pinned Node.js runtime (v22)
├── .env.example             # Template environment variables
├── .gitignore               # Repository ignore rules
├── README.md                # Project documentation
├── AGENTS.md                # Agent operational contract & boundaries
├── CLAUDE.md                # Development guide & workflow rules
├── crates/
│   ├── exodus-core/         # Domain models, outcome classifications, errors
│   ├── exodus-parser/       # Tree-sitter AST parser interfaces
│   ├── exodus-graph/        # Semantic graph (ESG) data structure & algorithms
│   ├── exodus-planner/      # Wave-based topological planner & risk weighting
│   ├── exodus-agent/        # Bounded agent repair loop & LLM harnesses
│   ├── exodus-transform/    # Construct transformation & code generation
│   ├── exodus-fallback/     # Fallback generator & migration debt handler
│   ├── exodus-verifier/     # Compiler verification & behavioral test runners
│   ├── exodus-eval/         # Evaluation benchmarks & metrics aggregation
│   └── exodus-cli/          # Command-line interface (`exodus` binary)
├── openwiki/                # OpenWiki knowledge base (OKF v0.2)
├── prompts/                 # Prompt templates for bounded agent repairs
├── fixtures/                # Test fixtures and legacy reference codebases
├── evaluation/              # Benchmark suites and evaluation datasets
├── docs/                    # Static visualizer exports and design specs
└── .exodus/                 # Runtime execution evidence, schemas, and debt logs
```

---

## Toolchain & Reproducibility

* **Rust**: `1.97+` (stable, 2021 edition, pinned via `rust-toolchain.toml`)
* **Node.js**: `22` (pinned via `.nvmrc` / Deno package management)
* **OpenWiki**: `0.4.3` (pinned in `package.json` and lockfile)

### Clean-Environment Commands

```bash
# Install toolchain dependencies
deno install

# Code quality and workspace tests
cargo fmt --check
cargo check --workspace
cargo test --workspace

# OpenWiki documentation workflows
npm run wiki:update
npm run wiki:visualize
```

---

## Next Implementation Phase

The initial repository, Rust workspace, and OpenWiki OKF v0.2 knowledge contract are fully established.

To start the next implementation phase (Phase 1: Tree-sitter Parser Frontend & ESG Construction):

```bash
cargo check -p exodus-parser -p exodus-graph
```
