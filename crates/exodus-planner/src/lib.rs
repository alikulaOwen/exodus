//! Migration planning engine, wave-based scheduling, and approval checkpoints.

use exodus_core::Result;
use exodus_graph::SemanticGraph;
use serde::{Deserialize, Serialize};

/// A single discrete step in a migration plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationStep {
    pub step_id: String,
    pub symbol_id: String,
    pub risk_score: u8,
    pub requires_human_approval: bool,
}

/// An ordered sequence of migration waves generated from semantic analysis.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub waves: Vec<Vec<MigrationStep>>,
}

/// Planner responsible for creating migration plans from the SemanticGraph.
pub struct MigrationPlanner;

impl MigrationPlanner {
    pub fn plan(_graph: &SemanticGraph) -> Result<MigrationPlan> {
        // Explicit extension point for graph-guided planning
        Ok(MigrationPlan::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planner_empty() {
        let graph = SemanticGraph::new();
        let plan = MigrationPlanner::plan(&graph).unwrap();
        assert!(plan.waves.is_empty());
    }
}
