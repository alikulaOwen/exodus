---
okf_version: "0.2"
type: workflow
status: planned
sources:
  - crates/exodus-verifier/src/lib.rs
  - crates/exodus-core/src/lib.rs
---

# Verification Lifecycle

[Index](index.md) | [Human Approval Model](human-approval-model.md) | [Evaluation Methodology](evaluation-methodology.md)

Verification ensures generated code matches behavioral specifications through a multi-tier gate:

## Stages
1. **Format Check**: `cargo fmt --check` ensures valid, readable syntax.
2. **Compilation Check**: `cargo check --workspace` verifies type safety, borrow checker compliance, and linkage.
3. **Unit Tests**: Synthesized and migrated unit tests executed via `cargo test`.
4. **Behavioral Equivalence**: Compare input/output behavior against recorded legacy test fixtures.

## Outcome State Machine
* **Passes All Stages** -> `Verified`
* **Compiles Cleanly, Tests Pending** -> `Compatible`
* **Contains Fallback Stubs** -> `Degraded`
* **Fails Compilation After Repair Loop** -> `Blocked`
