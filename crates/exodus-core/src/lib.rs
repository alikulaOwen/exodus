//! Core domain models, shared types, and outcome classifications for Project Exodus.

use serde::{Deserialize, Serialize};

/// Target and source programming languages supported by the migration engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Python,
    Rust,
    Custom(String),
}

/// The classification of a migration outcome for a symbol, unit, or codebase.
///
/// Reliable migration maximizes verified behavior while turning unresolved
/// semantics into explicit, measurable, and reviewable migration debt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MigrationOutcome {
    /// Fully compiled and validated against behavioral tests.
    Verified,
    /// Successfully transformed and compiles cleanly, awaiting behavioral tests.
    Compatible,
    /// Transformed using explicit fallback stubs or reduced semantics.
    Degraded,
    /// Could not be transformed; requires manual human intervention.
    Blocked,
}

/// Represents explicit migration debt incurred when fallback stubs or partial semantics are used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationDebt {
    pub symbol_id: String,
    pub description: String,
    pub reason: String,
    pub location: Option<String>,
}

/// Core error types across the Exodus engine.
#[derive(thiserror::Error, Debug)]
pub enum ExodusError {
    #[error("Parse error in {file}: {message}")]
    ParseError { file: String, message: String },
    #[error("Graph error: {0}")]
    GraphError(String),
    #[error("Planning error: {0}")]
    PlanningError(String),
    #[error("Transformation error: {0}")]
    TransformationError(String),
    #[error("Verification failure: {0}")]
    VerificationFailure(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Generic error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ExodusError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_classification() {
        assert_ne!(MigrationOutcome::Verified, MigrationOutcome::Degraded);
        assert_ne!(MigrationOutcome::Compatible, MigrationOutcome::Blocked);
    }
}
