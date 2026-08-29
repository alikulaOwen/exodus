# Project Exodus: Agent Trajectories & Trace Logs

This document contains representative trajectories capturing multi-agent interactions across the Exodus migration lifecycle.

---

## 1. Architecture Agent Trajectory (`.exodus/trajectories/architecture_agent.json`)

```json
{
  "agent": "ArchitectureAgent",
  "task": "Construct Exodus Semantic Graph (ESG) & Dependency Cycle Analysis",
  "input": {
    "target_directory": "fixtures/04_circular_dependency",
    "parser": "tree-sitter-python"
  },
  "tool_calls": [
    {
      "tool": "parse_ast",
      "args": { "files": ["user.py", "order.py"] },
      "result": { "modules": 2, "functions": 2, "imports": 2 }
    },
    {
      "tool": "tarjan_scc",
      "args": { "nodes": ["function::user::get_user_summary", "function::order::get_user_orders"] },
      "result": { "sccs": [["function::user::get_user_summary", "function::order::get_user_orders"]], "cycle_detected": true }
    }
  ],
  "outcome": {
    "status": "APPROVAL_REQUIRED",
    "risk_score": 75,
    "blocker_reason": "Cycle detected between user.py and order.py. Human approval required in .exodus/plan.json."
  }
}
```

---

## 2. Migration Agent Trajectory (`.exodus/trajectories/migration_agent.json`)

```json
{
  "agent": "MigrationAgent",
  "task": "Transform unsupported dynamic construct (eval)",
  "input": {
    "symbol": "dynamic_runtime::evaluate_runtime_code",
    "construct": "eval(expr_str)"
  },
  "decision": "FALLBACK_DEGRADED",
  "tool_calls": [
    {
      "tool": "emit_fallback_stub",
      "args": {
        "strategy": "TypedFailureStub",
        "symbol": "evaluate_runtime_code",
        "reason": "Dynamic reflection unsupported in static Rust"
      },
      "result": {
        "generated_code": "pub fn evaluate_runtime_code(expr_str: String) -> i64 {\n    todo!(\"Exodus Migration Debt: Dynamic reflection unsupported in static Rust\");\n}",
        "debt_recorded": true
      }
    }
  ]
}
```

---

## 3. Bounded Repair Agent Trajectory (`.exodus/trajectories/repair_agent.json`)

```json
{
  "agent": "RepairAgent",
  "task": "Fix compilation diagnostic E0425 in generated target",
  "iteration": 1,
  "max_iterations": 3,
  "diagnostic": "error[E0425]: cannot find value `result` in this scope",
  "action": "Injected `let mut result = Vec::new();` into block header",
  "verifier_feedback": "cargo check passed (0 errors)",
  "outcome": "VERIFIED_COMPATIBLE"
}
```
