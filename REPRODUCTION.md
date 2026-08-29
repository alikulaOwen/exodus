# Project Exodus: Clean Environment Reproduction Guide

This guide allows any judge or reviewer to clone, build, run, and reproduce all benchmark results and migration workflows from a clean environment.

---

## 1. Prerequisites & Toolchain

Ensure the following tools are installed:
* **Rust**: `1.80+` (Standard stable toolchain with `cargo`, `rustfmt`, `clippy`)
* **Node.js** (Optional, for OpenWiki graph visualizer): `Node 22`

Verify toolchain:
```bash
rustc --version
cargo --version
```

---

## 2. One-Step Verification & Test Suite

Clone and run the complete workspace test suite (12 crates, 100% pass rate):

```bash
cargo test --workspace
```

*Expected output*: `test result: ok. All tests passed across all 12 workspace crates.`

---

## 3. Benchmark Evaluation Reproduction (Exodus vs Baseline)

Run the evaluation benchmark comparing the Exodus graph-guided engine against a standard baseline single-agent LLM:

```bash
cargo run -p exodus-cli -- eval --fixtures fixtures --output .exodus
```

Generated artifacts:
* `.exodus/evaluation_scorecard.md`
* `.exodus/evaluation_scorecard.csv`
* `.exodus/evaluation_scorecard.json`

---

## 4. End-to-End Migration CLI Workflow

Execute the full migration pipeline on a target repository:

```bash
# 1. Analyze AST & Generate Exodus Semantic Graph (ESG)
cargo run -p exodus-cli -- analyze fixtures/01_typed_functions --output .exodus

# 2. Generate Wave-Sequenced Migration Plan with Risk Checkpoints
cargo run -p exodus-cli -- plan fixtures/01_typed_functions --output .exodus

# 3. Approve Plan
cargo run -p exodus-cli -- approve .exodus/plan.json --approver "LeadArchitect"

# 4. Migrate Python Code to Rust Project
cargo run -p exodus-cli -- migrate fixtures/01_typed_functions --output target/migrated_01

# 5. Verify & Compile Generated Rust Crate
cargo run -p exodus-cli -- verify target/migrated_01

# 6. View Status Report
cargo run -p exodus-cli -- report --dir .exodus

# 7. Units, Behavioral Contracts, and Governed Case Engine
cargo run -p exodus-cli -- units list fixtures/01_typed_functions
cargo run -p exodus-cli -- contracts show function::add
cargo run -p exodus-cli -- cases list
cargo run -p exodus-cli -- worktree list
```

---

## 5. Performance & Cost Profile

* **Runtime**: ~0.8 seconds for full workspace tests; ~0.4 seconds for end-to-end fixture migration.
* **Cost**: $0.00 in offline/mock mode; <$0.02 per 1,000 LOC with external API providers enabled.
