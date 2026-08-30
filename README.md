# Project Exodus: Graph-Guided, Agent-Assisted Legacy Code Migration Engine

> **Autonomous Polyglot Migration Platform with Explicit Migration Debt Accounting & Verifiable Behavioral Parity**

---

## 1. Problem, Intended User & Value Proposition

### Who is the Intended User?
* **Enterprise Modernization & Platform Engineering Teams**: Organizations maintaining mission-critical legacy applications (Python, legacy JavaScript/TypeScript, Java) burdened by technical debt, security vulnerabilities, or performance bottlenecks, needing to modernize into high-performance, memory-safe target languages (Rust, modern Go, Zig, TypeScript).
* **Software Architects & Migration Leads**: Engineers responsible for multi-package monorepos who need architectural visibility, topological scheduling, and guarantee of safety rather than black-box code dumps.

### The Current Bottleneck
When developers attempt automated migrations using raw LLMs (ChatGPT, Claude, Copilot) or naive ungrounded agents:
1. **Context Blindness & Cycle Breakdown**: Unstructured single-file prompts cannot perceive cross-module dependency graphs, failing completely on circular dependencies, shared types, and bottom-up dependency ordering.
2. **Hallucination & Silent Production Bugs**: When encountering unsupported dynamic constructs (e.g. `eval()`, runtime reflection, missing third-party C-bindings), raw LLMs hallucinate believable but non-functional dummy logic that passes superficial review but crashes in production.
3. **Destructive Changes & Zero Safety Verification**: Traditional tools mutate the active working tree directly and lack in-loop verification. Without automated compiler and behavioral test gate checks (`cargo check`, `cargo test`, differential testing), generated code fails to compile or breaks behavioral parity over 60% of the time.

### Why Solving It is Valuable in Practice
Project Exodus eliminates the modernization bottleneck by turning code migration into an **evidence-backed, graph-guided, compiler-verified engineering discipline**:
* **Topological Wave Scheduling**: Parses ASTs using Tree-sitter into an in-memory **Exodus Semantic Graph (ESG)** to detect cycles (Tarjan SCC) and migrate leaf libraries before callers.
* **Non-Destructive Git Worktree Isolation**: All mutations and builds occur in isolated Git worktrees (`.exodus/worktrees/`) with automated lease tracking and crash recovery.
* **Grounded Behavioral Contracts & Unit Verification**: Differential execution captures ground-truth runtime behavior from legacy code to verify target implementations before committing.
* **Governed Migration Case Engine**: Failures are fingerprinted into cross-repository migration cases with bounded repair (maximum 3 iterations).
* **Explicit Migration Debt Ledger**: Dynamic runtime constructs that cannot be statically mapped are emitted as explicit typed `todo!("Exodus Migration Debt: ...")` stubs recorded in `.exodus/fallbacks.json` rather than fabricated dummy logic.
* **Human-in-the-Loop Review**: Human approval gates are mandatory before breaking circular dependencies, approving plans, or executing high-impact transformations.

---

## 2. Improvement Changelog

Every iteration in Project Exodus is guided by empirical evidence and non-negotiable quality metrics:

| Iteration | Hypothesis & What We Built | Empirical Evidence & Verification | Decision & Learning |
|---|---|---|---|
| **Baseline** | Naive direct translation: single-pass function mapping without dependency graph awareness, type mapping, or class state grouping. | Baseline achieved **0.0% outcome pass rate** on multi-file dependencies and cyclic structures (`.exodus/evaluation_scorecard.md`). | Direct prompting fails on multi-file topologies and class hierarchies; architectural graph modeling is mandatory. |
| **Iteration 1: ESG & AST Engine** | Built Tree-sitter symbol extractor and the Exodus Semantic Graph (ESG) with Tarjan SCC cycle detection and wave planner (`exodus-parser`, `exodus-graph`, `exodus-planner`). | Correctly identified all cross-module imports and isolated cyclic clusters into human approval gates (`plan.json`). | **Kept**: Migrating dependencies in topological bottom-up order prevents cascading compiler breaks. |
| **Iteration 2: Deterministic Mapping & Fallback Debt** | Implemented deterministic type transformations and explicit fallback stubs (`todo!`) with `MigrationDebt` ledger (`exodus-transform`, `exodus-fallback`). | Whole-fixture compilation rate increased to **63.6%**; untranslatable dynamic reflection (`eval()`) safely compiled without hallucinated logic. | **Kept**: Refusing to hallucinate business logic and recording explicit migration debt is strictly safer than fake code. |
| **Iteration 3: Bounded In-Loop Repair** | Built bounded repair controller (`exodus-agent`) with maximum 3 auto-correction attempts per symbol against `rustc` JSON diagnostics. | Fixed common compiler diagnostics (missing mutability, missing derive traits, block scope scoping) within 1–2 iterations. | **Kept**: Bounding repairs to 3 iterations stops LLM runaway loops and prevents prompt token exhaustion. |
| **Iteration 4: Worktree Isolation & Governed Case Engine** | Built isolated Git worktrees (`exodus-worktree`) and structural fingerprinting case engine (`exodus-case`). | Zero contamination of host repository; compiler failures automatically generated reusable migration cases without symbol leaks. | **Kept**: Worktree leasing guarantees safety; structural case fingerprints allow genuine cross-repo pattern reuse. |
| **Iteration 5: Grounded Behavioral Unit Gate** | Implemented grounded unit verification contracts (`exodus-verifier`, `BehavioralContract`), differential Python capture, and atomic per-unit commits. | Verified 6 of 8 unit boundaries with honest classification (`Verified`, `Compatible`, `Degraded`, `Blocked`). | **Kept**: Unit-level contract execution catches functional regressions that whole-module compilation alone misses. |
| **Iteration 6: Interactive Harness & Host Toolchain Scaffolding** | Built interactive REPL harness with host version manager integration (`.tool-versions`, `.mise.toml`, language version files, `git init`), target isolation, and human thought approval (`exodus-toolchain`, `exodus-cli`). | Full workspace test suite passes (100% across 13 crates); standalone and monorepo output directories cleanly isolated outside source root. | **Kept (Current State)**: Gives developers a familiar, safe, interactive CLI experience with complete host toolchain autonomy. |

---

## 3. Main Failure Mode & Our Hot Take

### Observed Main Failure Mode
When legacy code relies on dynamic runtime metaprogramming or runtime string evaluation (e.g. `eval("x + y")`, dynamic `setattr` reflection, or unresolvable C-FFI pointer casting), static target compilation cannot guarantee equivalent behavior without embedding a full runtime interpreter. Naive AI coding tools hallucinate placeholder logic that silently passes tests but fails in production.

### Our Hot Take
> **"An agent that refuses to lie and emits explicit, measurable migration debt is 10x more valuable in enterprise production than an agent that pretends to translate 100% of the code with silent runtime bugs."**

---

## 4. Clean Environment Reproduction Guide

### Prerequisites
* **Rust**: `1.80+` stable (`cargo`, `rustc`, `rustfmt`, `clippy`)
* **Git**: `git` CLI (required for worktree isolation)
* **Python 3**: Python 3.10+ (for running differential Python baseline or grounding fixture contracts)

### Step 1: Clone & Verify Quality Gates
```bash
git clone https://github.com/exodus-migration/exodus.git
cd exodus

# Run all workspace test suites across 13 crates (100% passing)
cargo test --workspace

# Check formatting and clippy lints
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

### Step 2: Build Release Binary
```bash
cargo build --release
# Executable is located at ./target/release/exodus
```

### Step 3: Run Benchmark Suite (Exodus vs. Real Executed Baseline)
```bash
./target/release/exodus eval --fixtures fixtures --output .exodus
```
* **Expected Output**: Generates `.exodus/evaluation_scorecard.md`, `.csv`, and `.json` comparing Exodus against a real, executed non-graph baseline across whole-fixture and unit-level tiers.
* **Approximate Runtime**: ~35–45 seconds (runs full `cargo check` and `cargo test` subprocesses on all 11 benchmark fixtures).
* **Cost**: $0.00 USD (deterministic compiler and local AST analysis; $0 LLM cost in offline mode).

### Step 4: Run Interactive Agent Harness
```bash
./target/release/exodus
```
Inside the interactive REPL:
```text
exodus > /doctor                                     # Audit host toolchains (Node, Python, Go, Zig, Rust, Git)
exodus > /analyze fixtures/01_typed_functions        # Extract AST & build Exodus Semantic Graph (ESG)
exodus > /plan fixtures/01_typed_functions           # Compute dependency waves & approval checkpoints
exodus > /approve .exodus/plan.json                  # Review and approve migration plan
exodus > /migrate fixtures/01_typed_functions --gated # Execute isolated worktree gated migration
exodus > convert crates/exodus-fallback to zig       # Natural language multi-language conversion
exodus > /exit                                       # Clean session exit (:q, exit, quit, Ctrl+C)
```

---

## 5. Agent Instructions, Prompts & Safety Boundaries

Project Exodus structures multi-agent coordination using strict role-based prompts and invariant contracts:

### Non-Negotiable Invariants
1. **Strict Metric Honesty**: Never fabricate verification statuses or benchmark baselines. All outcomes must derive from actual subprocess execution (`cargo check`, `cargo test`, diff execution).
2. **Outcome Classification Tiers**:
   * `Verified`: Passes compiler checks AND behavioral test contracts.
   * `Compatible`: Compiles cleanly without stubs, awaiting test suite validation.
   * `Degraded`: Relies on explicit `todo!` fallback stubs; recorded as `MigrationDebt`.
   * `Blocked`: Fails compilation or transformation after bounded repair attempts.
3. **Atomic Boundaries**: Classes and their methods group into a single verification unit.
4. **Symbol-Agnostic Structural Hashing**: Case engine fingerprints hash graph topology and failure categories without symbol names.
5. **Bounded Repair**: Maximum **3 iterations** per symbol before emitting fallback stubs.
6. **Human Approval Gate**: Destructive actions, plan approvals, and cycle breaking require human authorization.

### Agent Prompts & System Roles

#### 1. Migration Agent (`exodus-agent::LlmMigrationEngine`)
```text
You are the Exodus Migration Agent specializing in translating legacy source code into modern, idiomatic target code.
You must adhere strictly to:
1. Idiomatic Target Design: Use standard idioms, pattern matching, error handling (Result/Option), and memory ownership.
2. Grounded Semantics: Never guess or invent dummy logic for unsupported reflection. If a construct cannot be translated, emit an explicit fallback stub: todo!("Exodus Migration Debt: <reason>").
3. Preserve Symbol Interfaces: Retain expected signatures and public entrypoints.
```

#### 2. Bounded Repair Agent (`exodus-agent::BoundedAgent`)
```text
You are the Exodus Bounded Repair Agent. Your goal is to fix compiler diagnostic errors reported by rustc.
You have a strict maximum budget of 3 iterations.
Input: Current source code + rustc JSON diagnostic errors.
Output: Repaired source code addressing only the localized error without introducing regressions.
```

---

## 6. Solution Video Script (5 Minutes)

* **[0:00 - 0:45] Problem & The Intended User**: Introduce the challenge of migrating legacy codebases to Rust/Go/Zig. Demonstrate how naive LLM prompts fail on dependency cycles and dynamic reflection.
* **[0:45 - 1:30] Baseline Failure Demonstration**: Show the unguided baseline failing on circular imports and dynamic `eval()`, producing broken code.
* **[1:30 - 2:30] Project Exodus Architecture**: Walk through the 13-crate architecture: Tree-sitter parser $\to$ ESG semantic graph $\to$ Tarjan SCC cycle detector $\to$ Git worktree isolation $\to$ Bounded repair verifier $\to$ Governed Case Engine.
* **[2:30 - 3:45] Live E2E Execution & Interactive Harness**: Run `./target/release/exodus`, demonstrate `/doctor` toolchain audit, wave planning with human approval prompt, gated worktree migration with atomic commits, and natural language translation (`convert exodus-fallback to zig`).
* **[3:45 - 4:15] Improvement Changelog & Removed Experiment**: Review the progression from Baseline to Iteration 6. Highlight the experiment removed: trying to generate raw `unsafe` Rust for dynamic types was eliminated in favor of explicit typed fallback stubs.
* **[4:15 - 5:00] Hot Take & Conclusion**: Close with the core thesis: metric honesty, worktree safety, and explicit migration debt deliver production-grade trust.

---

## 7. Submission Deliverables & Directory Structure

* **`crates/`**: 13 modular, production-tested Rust crates (`exodus-core`, `exodus-parser`, `exodus-graph`, `exodus-planner`, `exodus-agent`, `exodus-transform`, `exodus-fallback`, `exodus-verifier`, `exodus-case`, `exodus-worktree`, `exodus-eval`, `exodus-kernel`, `exodus-toolchain`, `exodus-cli`).
* **`openwiki/`**: Complete Open Knowledge Format v0.2 wiki knowledge base (`openwiki/index.md`, `openwiki/human-approval-model.md`, `openwiki/user-and-migration-bottleneck.md`, etc.).
* **`fixtures/`**: 11 synthetic and real-world legacy code fixtures with grounded contracts.
* **`REPRODUCTION.md`**: Clean environment reproduction guide.
* **`VIDEO_SCRIPT.md`**: 5-minute storyboard and video walkthrough script.
* **`TRAJECTORIES.md`**: Representative agent execution traces and tool interaction logs.
