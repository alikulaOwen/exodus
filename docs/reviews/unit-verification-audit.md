---
type: audit-report
title: Unit-Level Migration & Behavioral-Contract Verification — Audit
status: active
okf_version: "0.2"
---

# Unit Verification Audit — Project Exodus

This document is written in two parts. The **Pre-Change** section below is preserved evidence
captured before any implementation work in this pass and must never be rewritten. A **Post-Change**
section is appended at the end once implementation is complete, so before/after can be compared
directly.

---

## Pre-Change Section (do not edit after implementation begins)

### Repository state at audit time

- HEAD commit: `55396d5688e42dcf324a87b85f2500af1d6e266f` ("feat: Add foundational documentation and
  structure for Project Exodus"), 2026-08-29 08:53:28 +0300.
- `git status --short`: 22 tracked files modified (`Cargo.lock`, `Cargo.toml`, `README.md`, every
  `crates/*/src/lib.rs` and `crates/*/Cargo.toml` except `exodus-case`/`exodus-worktree`,
  `docs/openwiki-visualizer/graph.json`), plus untracked: `.exodus/*` artifacts, `REPRODUCTION.md`,
  `TRAJECTORIES.md`, `VIDEO_SCRIPT.md`, `crates/exodus-case/`, `crates/exodus-worktree/`,
  `fixtures/*` (11 fixture directories), `schemas/`.
- `git diff --stat`: 24 files changed, 4383 insertions(+), 324 deletions(-) relative to HEAD.
- Toolchain versions: `rustc 1.98.0`, `cargo 1.98.0`, `git 2.55.0`, `node v26.7.0`, `deno 2.9.5`.
- Workspace crates (12, confirmed via root `Cargo.toml`): `exodus-core`, `exodus-parser`,
  `exodus-graph`, `exodus-planner`, `exodus-agent`, `exodus-transform`, `exodus-fallback`,
  `exodus-verifier`, `exodus-case`, `exodus-worktree`, `exodus-eval`, `exodus-cli`.

**Operational note — concurrent external mutation.** A separate agent tool, Antigravity
(process `/home/alikula/.gemini/bin/agy --hub --hub-port=34789 --app_data_dir=antigravity
--add-dir=/home/alikula/Documents/projects/exodus`, PID 1020647, started 08:54), was actively
rewriting this repository during the early part of this audit. Three independent read-only
exploration passes each caught files (`crates/exodus-core/src/lib.rs`, `crates/exodus-case/src/lib.rs`,
`crates/exodus-worktree/src/lib.rs`) changing content and line counts between successive reads in the
same session, and `cargo build`/`cargo clippy`/`cargo test` gave materially different results seconds
apart (unresolved imports in one run, missing-method errors in the next, then a clean pass). The user
confirmed Antigravity's implementation work had stopped. A final targeted verification pass still found
the `agy` process listed in `ps`, but the repository content itself was internally consistent across
repeated command reruns at that point, and remained byte-identical (`git status --short` /
`git diff --stat` unchanged) immediately before implementation began. **The snapshot below reflects
that final stable state**, not any of the earlier transient/contradictory reads.

### Commands executed and pre-change results

```bash
$ cargo fmt --check
# exit 0 — clean, no files need formatting

$ cargo clippy --workspace --all-targets -- -D warnings
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.07s
# exit 0 — 0 warnings, 0 errors

$ cargo build --workspace
# Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.57s
# exit 0 — all 12 workspace members compile

$ cargo test --workspace
# 16 tests total across 11 crates + cli binary, 16 passed, 0 failed, 0 ignored
# per-crate: exodus-agent 2, exodus-case 1, exodus-core 1, exodus-eval 1, exodus-fallback 1,
#            exodus-graph 2, exodus-parser 2, exodus-planner 2, exodus-transform 2,
#            exodus-verifier 1, exodus-worktree 1, exodus-cli 0
# exit 0

$ cargo run -p exodus-cli -- eval --fixtures fixtures --output .exodus
# Total Fixtures: 11 | Exodus Behavioral Pass Rate: 90.9% (Baseline: 27.3%)
# Compilation Rate: 100.0% (Baseline: 36.4%) | Migration Debts: 1 | Human Interventions: 0
# 10/11 fixtures Verified; 1 (10_deliberately_untranslatable_reflection) Degraded, tests FAIL, 1 debt
# exit 0
```

Documented end-to-end workflow (`analyze → plan → approve → migrate → verify → report`) is real and
runs; `Verifier` genuinely shells out to `rustfmt` / `cargo check --message-format=json` / `cargo test`
(`crates/exodus-verifier/src/lib.rs`, `run_formatter`/`run_cargo_check`/`run_cargo_tests`). No documented
command in `README.md`/`REPRODUCTION.md` was found to be missing or non-existent, though several
produce numbers that do not match what they claim (see Critical Findings).

### Requirement matrix

| Requirement | Status | Code Evidence | Test Evidence | Gap | Required Change |
|---|---|---|---|---|---|
| ESG identifies functions/methods/classes | Implemented | `crates/exodus-graph/src/lib.rs:11-21` (`NodeKind`), `:391-466` (populated from `ClassDef`/`FunctionDef`) | `exodus-graph` unit tests (2) | Only a generic `Module` kind exists; no distinct "utility module" classification | None required — cosmetic only |
| ESG unit dependencies | Implemented | `dependencies_of`/`dependents_of`, `:123-155` | covered indirectly by graph tests | — | — |
| Strongly connected components | Implemented | `find_sccs` (real Tarjan, `:158-243`), `detect_cycles` (`:246`) | covered by graph tests | — | — |
| Smallest independently compilable clusters | Missing | SCCs computed but never grouped into an atomic verification boundary; `exodus-planner` flags cyclic nodes for approval but still plans them individually | none | No cluster abstraction anywhere in the codebase | Add `VerificationCluster`/`verification_units()` (Milestone 1) |
| Stable unit identities | Partial | IDs are `format!("{kind}::{qualified_name}")` (`:392,423,447`) — deterministic but with no content hash or explicit re-parse-stability guarantee | none | ID changes only if the qualified name changes; a body-only edit is invisible to the ID | Add content-hash component (Milestone 1) |
| Source spans/evidence | Implemented | `SourceEvidence`, tree-sitter-backed (`crates/exodus-parser/src/lib.rs:123-137`) | parser tests | — | — |
| Behavioral contract sourcing (source tests / golden fixtures / differential exec / invariants / signatures / human-approved) | Missing | `BehavioralContract`/`BehavioralAssertion` (`crates/exodus-core/src/lib.rs:161,171`) instantiated **only** in one unit test; no production code path populates one | none | Fixtures contain bare `.py` files with zero oracle/expected-output data; `.exodus/contracts/` empty on disk | Ground contracts per Milestones 2–3 |
| `contracts show` fabricating placeholder contracts | Incorrect | `crates/exodus-cli/src/main.rs:456-472` — hardcoded `source_signature: "fn() -> int"` etc. returned whenever the (always-absent) real file is missing | none | Directly violates "never invent expected outputs from a signature" | Remove fallback fabrication; report "not verified" (Milestone 8) |
| Unit harness generation | Missing | `Verifier::scaffold_target_crate` accepts `behavioral_tests: Option<&str>` but `Commands::Migrate` always passes `None` (`main.rs:313`) | none | No harness is ever generated or run through the CLI pipeline | Milestone 4 |
| Per-unit migration gate (9-step sequence) | Missing | No orchestration exists that resolves deps → retrieves cases → generates unit → compiles → runs contract → commits/localizes → repairs → re-verifies | none | `migrate`/`verify` operate on whole modules/repos, not units | Milestone 5 |
| Verification hierarchy (unit/component/module/repo/e2e reported separately) | Missing | `report`/`eval` expose one rolled-up `MigrationOutcome` per fixture | none | No separate tiers anywhere | Milestone 9 |
| Contract artifacts under `.exodus/contracts/<unit-id>/` | Missing | Directory exists but is empty; nothing writes to it except the CLI's fabricated fallback read path | none | — | Milestone 5/8 |
| Case Engine capturing unit failures with subgraph/observations | Partial | `MigrationCase` (`crates/exodus-case/src/lib.rs:37-57`) has no `unit_id`, no ESG subgraph, no source/target observation fields, only a free-text `failure_description` + `Option<String> compiler_diagnostic` | 1 passing unit test for lifecycle, not integration | `capture_failure` has no caller anywhere except its own test | Milestone 6 |
| Structural case fingerprint | Incorrect | `compute_fingerprint` (`:86-104`) hashes failure-category + node/edge **counts** (not topology) + lowercased symbol name via a djb2-style function dishonestly named `md5_or_simple_hash` (`:231-237`, not MD5) | 1 test | Two structurally different failures with same category/counts/name collide | Milestone 6 |
| Worktree isolation for all mutation/verification | Incorrect | `WorktreeManager` exists and creates real `git worktree add --lock -b exodus/<task>/migration` leases (`crates/exodus-worktree/src/lib.rs:103-162`), but `Commands::Migrate`/`Commands::Verify` never call it — all mutation/compilation happens directly in the primary working tree | 1 unit test on the manager in isolation | Master-prompt requirement "never mutate the user's primary working tree" is violated in the actual pipeline today | Milestone 7 |
| Atomic per-unit worktree commits | Missing | Zero `git commit` invocations anywhere in the codebase (`grep -rn '"commit"' crates/` finds only unpopulated struct field names) | none | — | Milestone 7 |
| Dirty worktrees preserved, not force-deleted | Incorrect | `cleanup_lease` (`:224-238`) unconditionally `fs::remove_dir_all`s; `WorktreeStatus::DirtyReviewRequired`/`Recoverable` exist in the enum but are never set or checked | none | `Discard` and `Cleanup` CLI commands behave identically | Milestone 7 |
| Worktree lease failure handling | Incorrect | `create_lease` logs a `tracing::warn!` but still persists a lease with `status: Active` when `git worktree add` itself fails (`:118-161`) | none | Callers cannot distinguish a real isolated worktree from this silent fallback | Milestone 7 |
| CLI: `units list` | Implemented | `main.rs:426-437`, walks real `SemanticGraph` | manual run confirmed | — | — |
| CLI: `units show <unit-id>` | Missing (stub) | `main.rs:440` — `println!` echoing the ID only | none | No lookup performed | Milestone 8 |
| CLI: `units verify <unit-id>` | Missing (stub) | `main.rs:443` — unconditional hardcoded success print | none | — | Milestone 8 |
| CLI: `contracts show <unit-id>` | Incorrect (see above) | `main.rs:447-475` | none | — | Milestone 8 |
| CLI: `contracts verify <unit-id>` | Missing (stub) | `main.rs:477` — unconditional hardcoded success print | none | — | Milestone 8 |
| CLI: `cases test --mode replay\|verify` | Missing (stub) | `main.rs:530-536` — `println!("...100% PASS.")` regardless of any case content, `engine.list_cases()` never even called | none | Direct contradiction of the project's own "never fabricate verification statuses" invariant | Milestone 8 |
| CLI: `cases search` | Incorrect | `main.rs:504-506` — hardcodes `FailureCategory::AsyncCallbackSemantics`, ignoring the user's query for filtering | none | — | Milestone 8 |
| CLI `--json` output | Partial | Global flag exists and is honored on `analyze`/`plan`/`approve`/`migrate`/`verify`/`eval`; not yet meaningful on the stubbed `units`/`contracts`/`cases` commands | none | — | Milestone 8/11 |
| Repair loop bounded to 3 attempts | Partial (unwired) | `BoundedAgent`/`AgentBounds::default` (`crates/exodus-agent/src/lib.rs:20-29,169-241`) implement a real bounded loop; `Verifier::verify_and_repair` (`crates/exodus-verifier/src/lib.rs:162-253`) is the only caller and has **no callers of its own** outside its definition | 2 agent-crate tests | `exodus verify` bypasses repair entirely | Milestone 5 |
| Typed fallback stubs never lie | Implemented | `crates/exodus-fallback/src/lib.rs:89` — `todo!("Exodus Migration Debt: {}")`, by design, not a code stub | 1 test | — | — |
| Evaluation: unit/component/module/repo rates reported separately | Missing | `crates/exodus-eval/src/lib.rs` has one blended `exodus_pass_rate_pct`/`exodus_compilation_rate_pct` per fixture | 1 test (aggregate only) | No grounded-oracle-coverage or failure-localization-accuracy metric exists anywhere in the repo | Milestone 10 |
| Evaluation: baseline is a real, measured run | Incorrect | `crates/exodus-eval/src/lib.rs:92-96` hardcodes `(baseline_compiled, baseline_tests_passed)` by matching the **fixture directory name string** — no baseline agent is ever executed | none | README/REPRODUCTION's "30% baseline" headline is fabricated, not measured | Milestone 10 |
| Evaluation: Exodus test pass derived from real test execution | Incorrect | `exodus_tests_passed` (`:98-102`) is inferred purely from the transform-stage `MigrationOutcome` enum, never from running `cargo test` on generated code | none | — | Milestone 10 |
| Versioned JSON Schemas for contracts/cases/merge-proposals | Partial | `schemas/behavioral-contract.schema.json`, `schemas/case.schema.json`, `schemas/merge-proposal.schema.json` exist and structurally match their Rust types | none | No `schemas/verification.json`; no schema-validation code path anywhere in the Rust source (schemas exist but are never loaded/checked at runtime) | Milestone 2 |
| `.exodus/schemas/README.md` accuracy | Incorrect | Claims 4 files (`graph.schema.json`, `plan.schema.json`, `debt.schema.json`, `report.schema.json`) that do not exist anywhere in the repo | none | Stale/aspirational documentation | Milestone 12 |
| Fixture set covers the 10 required categories + two-run demo | Implemented (structurally) | 11 fixture directories present matching every required category, plus `two_run_demo/{repo_a,repo_b}` | eval run confirms all 11 parse/transform | Each fixture is bare `.py` source with **no** oracle/expected-output/manifest files | Milestone 3 |
| Two-run case-learning experiment | Missing | `fixtures/two_run_demo/{repo_a,repo_b}` exist on disk, but `CaseEngine` is never invoked from the real migrate/verify path, so no case is ever captured, promoted, or retrieved for repo B | none | — | Out of scope for this pass — flagged as remaining work |
| OpenWiki OKF v0.2 validity | Implemented | 22 concept pages under `openwiki/`, all sampled pages carry valid `okf_version: "0.2"` + `type` + `status` frontmatter; every sampled page is honestly `status: planned` | none | Docs currently under-claim relative to what's implemented (a safe direction), will need selective promotion | Milestone 12 |
| Trajectories are real captured runs, not hand-authored examples | Incorrect | `TRAJECTORIES.md` cites `.exodus/trajectories/architecture_agent.json`, `migration_agent.json`, `repair_agent.json` — **none exist**. Only `.exodus/trajectories/trajectory_sample.jsonl` exists (907 bytes, filename literally says "sample", different field schema than what `TRAJECTORIES.md` shows) | none | Documentation describes artifacts that were never generated | Milestone 12 |
| Fallback ledger consistency | Incorrect | `.exodus/fallbacks.json` is empty `[]` even though `evaluation_scorecard.json` reports `debts_recorded: 1` for fixture 10 — `eval`/`report` never write to the fallback ledger, only `migrate` does, and `migrate` was not run against the full fixture set in this snapshot | none | — | Milestone 12 (doc) / note in Milestone 10 (metrics) |
| No real LLM provider | Missing (acknowledged, likely out of scope) | `AgentProvider` trait has exactly one implementer, `MockAgentProvider` (`crates/exodus-agent/src/lib.rs:71`); no `reqwest`/HTTP client/Anthropic/OpenAI integration anywhere despite `.env.example` listing provider keys | 2 tests against the mock | REPRODUCTION.md's "$0.02/1000 LOC with OpenAI/Anthropic enabled" claim is unverifiable from source | Deferred — no credentials available in this environment |

### Critical findings (summary)

1. Core pipeline (parse → ESG → plan → transform → cargo-based verify) is genuinely implemented and
   should be preserved as-is.
2. Everything downstream of "verify a single unit's behavior against grounded evidence" is either
   unimplemented or actively fabricates its output: contracts are invented from placeholders, `cases
   test` always prints "100% PASS", the eval baseline is a hardcoded name-match, and worktree isolation
   / atomic commits / repair-loop wiring / case-capture wiring do not participate in the real
   `migrate`/`verify` pipeline at all despite each having a real, tested, but disconnected
   implementation sitting in its own crate.
3. Several shipped documents (`TRAJECTORIES.md`, `.exodus/schemas/README.md`, `REPRODUCTION.md`'s
   fixture-count table) describe artifacts or numbers that do not exist or do not match the current
   fixture set — these are corrected in Milestone 12, not before, so the corrections can cite the
   final implemented state rather than guessing ahead of it.
4. No `todo!()`/`unimplemented!()` markers exist as literal unfinished-code placeholders in the core
   pipeline crates — the gap is architectural (real subsystems that are never wired together), not
   missing code inside otherwise-complete functions.

### Evidence paths referenced above

- `crates/exodus-core/src/lib.rs`, `crates/exodus-graph/src/lib.rs`, `crates/exodus-parser/src/lib.rs`,
  `crates/exodus-planner/src/lib.rs`, `crates/exodus-transform/src/lib.rs`,
  `crates/exodus-verifier/src/lib.rs`, `crates/exodus-agent/src/lib.rs`,
  `crates/exodus-fallback/src/lib.rs`, `crates/exodus-case/src/lib.rs`,
  `crates/exodus-worktree/src/lib.rs`, `crates/exodus-eval/src/lib.rs`,
  `crates/exodus-cli/src/main.rs`
- `schemas/behavioral-contract.schema.json`, `schemas/case.schema.json`,
  `schemas/merge-proposal.schema.json`
- `.exodus/architecture.json`, `.exodus/fallbacks.json`, `.exodus/graph.json`,
  `.exodus/evaluation_scorecard.{json,md,csv}`, `.exodus/trajectories/trajectory_sample.jsonl`,
  `.exodus/schemas/README.md`
- `fixtures/01_typed_functions/` … `fixtures/10_deliberately_untranslatable_reflection/`,
  `fixtures/two_run_demo/{repo_a,repo_b}`
- `openwiki/` (22 concept pages), `openwiki/INSTRUCTIONS.md`
- `README.md`, `REPRODUCTION.md`, `TRAJECTORIES.md`, `VIDEO_SCRIPT.md`

### Implementation plan

See `/home/alikula/.claude/plans/you-are-an-independent-async-hoare.md` for the full milestone plan
(M0–M12) approved for this pass. In summary, in dependency order: stable unit identities and cluster
boundaries (M1) → contract model/provenance/schemas (M2) → grounded oracle data for a representative
fixture subset via differential execution (M3) → unit harness generation (M4) → the per-unit
verification gate that connects transform/verify/case/agent (M5) → Case Engine field/fingerprint fixes
and real wiring (M6) → worktree wiring and real atomic commits (M7) → CLI de-stubbing (M8) →
verification-hierarchy reporting (M9) → evaluation metrics overhaul with a real baseline (M10) →
required test coverage (M11) → OpenWiki and documentation corrections (M12).

Contract grounding in this pass is concentrated on five representative fixtures
(`01_typed_functions`, `02_class_conversion`, `03_module_dependency`, `04_circular_dependency`,
`08_async_function`) rather than fabricating coverage across all eleven; remaining fixtures are
reported honestly as ungrounded rather than silently claimed as covered.

---

<!-- POST-CHANGE SECTION APPENDED BELOW ONCE IMPLEMENTATION IS COMPLETE -->
