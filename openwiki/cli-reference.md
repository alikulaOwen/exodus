---
okf_version: "0.2"
type: reference
status: planned
sources:
  - crates/exodus-cli/src/main.rs
  - Cargo.toml
---

# CLI Reference

[Index](index.md) | [Reproduction Guide](reproduction-guide.md) | [Artifact Schemas](artifact-schemas.md)

The `exodus` command-line binary provides full workflow control:

## Commands
* `exodus parse --path <dir>`: Parses legacy repository into AST symbols.
* `exodus graph --path <dir>`: Constructs the Exodus Semantic Graph.
* `exodus plan --path <dir>`: Generates wave-based migration schedule into `.exodus/plan.json`.
* `exodus migrate --plan <path>`: Executes approved migration plan.
* `exodus verify --target <dir>`: Runs formatters, compilers, and tests on migrated code.
* `exodus report --evidence <dir>`: Outputs comprehensive outcome metrics and debt summaries.
