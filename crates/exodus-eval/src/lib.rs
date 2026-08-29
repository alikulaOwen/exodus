//! Evaluation metrics, benchmarking harnesses, and migration debt analytics.

use exodus_core::MigrationOutcome;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Summary metrics across an entire migration evaluation.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EvaluationSummary {
    pub total_symbols: usize,
    pub verified_count: usize,
    pub compatible_count: usize,
    pub degraded_count: usize,
    pub blocked_count: usize,
    pub outcome_breakdown: HashMap<String, usize>,
}

/// Evaluator harness for migration runs.
pub struct Evaluator;

impl Evaluator {
    pub fn calculate_metrics(outcomes: &[MigrationOutcome]) -> EvaluationSummary {
        let mut summary = EvaluationSummary {
            total_symbols: outcomes.len(),
            ..Default::default()
        };

        for outcome in outcomes {
            match outcome {
                MigrationOutcome::Verified => summary.verified_count += 1,
                MigrationOutcome::Compatible => summary.compatible_count += 1,
                MigrationOutcome::Degraded => summary.degraded_count += 1,
                MigrationOutcome::Blocked => summary.blocked_count += 1,
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluator_metrics() {
        let outcomes = vec![
            MigrationOutcome::Verified,
            MigrationOutcome::Compatible,
            MigrationOutcome::Degraded,
            MigrationOutcome::Blocked,
        ];
        let summary = Evaluator::calculate_metrics(&outcomes);
        assert_eq!(summary.total_symbols, 4);
        assert_eq!(summary.verified_count, 1);
        assert_eq!(summary.compatible_count, 1);
        assert_eq!(summary.degraded_count, 1);
        assert_eq!(summary.blocked_count, 1);
    }
}
