---
okf_version: "0.2"
type: reference
status: planned
sources:
  - openwiki/INSTRUCTIONS.md
  - crates/exodus-fallback/src/lib.rs
---

# Known Limitations

[Index](index.md) | [Fallback and Repair Layer](fallback-and-repair-layer.md) | [Evaluation Methodology](evaluation-methodology.md)

Documented limits and unsupported dynamic constructs:

## Unsupported Dynamic Semantics
* **Dynamic Code Execution**: `eval()`, `exec()`, and dynamic module loading require manual conversion or fallback stubs.
* **Dynamic Metaclasses / Monkey-Patching**: Dynamic runtime modification of classes is mapped to explicit trait objects or classified as `Blocked`.
* **Complex Cyclic Macros**: Multi-crate circular macro dependencies require human intervention.
