---
okf_version: "0.2"
type: invariant
status: planned
sources:
  - openwiki/INSTRUCTIONS.md
  - crates/exodus-planner/src/lib.rs
---

# Human Approval Model & Interactive Review Gate

[Index](index.md) | [Migration Planner](migration-planner.md) | [Agent Boundaries](agent-boundaries.md) | [Security Boundaries](security-boundaries.md)

Human oversight is a non-negotiable invariant in the Exodus migration engine. Consequential actions—such as breaking circular dependency cycles, deleting files, applying irreversible AST transformations, or running LLM synthesized code mutations—are governed by explicit human approval gates.

## Core Invariant: Qualified Human Reviewer

> **Rule**: Never perform destructive changes, circular dependency breaking, or irreversible transformations without generating an approval request in `.exodus/plan.json` or prompting the interactive human reviewer in the active harness session.

## Checkpoint Levels & Interactive Review Flow

1. **Interactive Thought & Idea Review (`claude-code` & `agy` style)**:
   * During interactive migration sessions, before executing high-impact actions or complex transform waves, the agent pauses with a structured proposal:
     ```text
     🤔 Agent Thought Process / Proposed Action:
        Topic:    Circular Dependency Decoupling (user.py <-> order.py)
        Details:  Extract shared OrderSummary struct into common interface layer to break cyclic import.

     Approve this action? [Y/n/e(dit)]:
     ```
   * **`Y` (Approve)**: Proceeds with transformation and bounded verification.
   * **`N` (Reject/Defer)**: Aborts or halts the operation, preserving untouched state.
   * **`E` (Edit)**: Allows the user to inject manual constraints, guidelines, or prompt adjustments.

2. **Plan Approval Gate (`exodus approve`)**:
   * Topologically generated multi-wave plans containing risk flags or cycle breaks are saved to `.exodus/plan.json` in an `ApprovalPending` status (`is_approved: false`).
   * Execution is strictly blocked until an authorized reviewer signs off:
     ```bash
     exodus approve .exodus/plan.json --approver "LeadArchitect"
     ```

3. **Fallback & Debt Acceptance**:
   * For unresolvable dynamic reflection (e.g. `eval()`, runtime metaprogramming), the engine emits an explicit typed fallback stub (`todo!("Exodus Migration Debt: ...")`) and logs the debt to `.exodus/fallbacks.json`.

4. **Git Worktree Isolation & Merge Proposals**:
   * All mutations occur inside isolated Git worktrees (`.exodus/worktrees/`).
   * Upon successful per-unit verification, Exodus generates a non-destructive `MergeProposal` that requires final human sign-off before merging into the main branch.

## Session Control Flow & Keyboard Shortcuts

The interactive harness uses standard control-flow shortcuts for seamless operation:

* **Exit / Quit**: `exit`, `quit`, `/exit`, `/quit`, `:q`, `:wq`, `q`, `/bye`, `bye`
* **Interrupt**: `Ctrl+C` cleanly aborts the running command and returns to the prompt without corrupting state.
* **Directory Navigation**: `cd <path>` or `/cd <path>` changes the active working context.
* **Target Output**: `/out [path]`, `/output [path]`, `/target [path]` inspects or sets custom isolated target directories.

