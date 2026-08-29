---
okf_version: "0.2"
type: evaluation
status: planned
sources:
  - crates/exodus-eval/src/lib.rs
  - crates/exodus-core/src/lib.rs
---

# Evaluation Methodology

[Index](index.md) | [Verification Lifecycle](verification-lifecycle.md) | [Artifact Schemas](artifact-schemas.md)

Exodus benchmarks modernization success on verifiable outcomes rather than raw lines of translated text:

## Evaluation Metrics
* **Verification Rate**: Percentage of symbols achieving `Verified` status (compilation + behavioral tests pass).
* **Compatibility Rate**: Percentage of symbols compiling cleanly without stubs (`Compatible`).
* **Degradation Ratio**: Explicit `MigrationDebt` instances vs total transformed symbols.
* **Blocker Ratio**: Symbols requiring manual human engineering (`Blocked`).
* **Bounded Repair Efficiency**: Success rate of agentic repair loops within the 3-iteration bound.
