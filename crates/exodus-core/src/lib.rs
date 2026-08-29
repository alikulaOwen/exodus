//! Core domain models, shared types, outcome classifications, and contracts for Project Exodus.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Target and source programming languages supported by the migration engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Python,
    Rust,
    Custom(String),
}

/// The classification of a migration outcome for a symbol, unit, or codebase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MigrationOutcome {
    Verified,
    Compatible,
    Degraded,
    Blocked,
}

impl std::fmt::Display for MigrationOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Verified => write!(f, "Verified"),
            Self::Compatible => write!(f, "Compatible"),
            Self::Degraded => write!(f, "Degraded"),
            Self::Blocked => write!(f, "Blocked"),
        }
    }
}

/// Risk level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Precise position in a source file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub byte_offset: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize, byte_offset: usize) -> Self {
        Self {
            line,
            column,
            byte_offset,
        }
    }
}

/// Byte and line-column span in a source file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceSpan {
    pub file_path: PathBuf,
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl SourceSpan {
    pub fn new(file_path: impl Into<PathBuf>, start: SourceLocation, end: SourceLocation) -> Self {
        Self {
            file_path: file_path.into(),
            start,
            end,
        }
    }

    pub fn point(file_path: impl Into<PathBuf>, line: usize, column: usize) -> Self {
        let loc = SourceLocation::new(line, column, 0);
        Self {
            file_path: file_path.into(),
            start: loc.clone(),
            end: loc,
        }
    }
}

/// Traceable evidence back to source files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceEvidence {
    pub span: SourceSpan,
    pub snippet: Option<String>,
    pub ast_node_kind: String,
}

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// Diagnostic emitted during parsing, analysis, compilation, or verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub severity: Severity,
    pub span: Option<SourceSpan>,
    pub suggestion: Option<String>,
}

impl Diagnostic {
    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        span: Option<SourceSpan>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity: Severity::Error,
            span,
            suggestion: None,
        }
    }

    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        span: Option<SourceSpan>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity: Severity::Warning,
            span,
            suggestion: None,
        }
    }
}

/// Represents explicit migration debt incurred when fallback stubs or partial semantics are used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MigrationDebt {
    pub symbol_id: String,
    pub description: String,
    pub reason: String,
    pub location: Option<SourceSpan>,
    pub fallback_strategy: String,
    pub confidence_score: f32,
    pub requires_human_review: bool,
}

/// Provenance of a behavioral-contract assertion, in the strength order the master spec requires:
/// a type signature alone never establishes complete expected behavior, so it must never be the
/// sole basis for an `expected` value (see [`OracleType::is_grounded`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleType {
    /// Existing source tests/fixtures — strongest evidence.
    SourceTest,
    /// Executable examples and approved golden files.
    GoldenFixture,
    /// The legacy unit was actually executed with controlled inputs and its output captured.
    DifferentialExecution,
    /// A declared precondition/postcondition/invariant or documented error behavior.
    DeclaredInvariant,
    /// Type signature, annotation, or docstring alone — describes shape, not complete behavior.
    TypeSignature,
    /// Synthesized with no stronger oracle available, and explicitly approved by a human reviewer.
    HumanApprovedSynthesized,
}

impl OracleType {
    /// 1 = strongest evidence, matching the master-prompt's strength ordering.
    pub fn strength_rank(&self) -> u8 {
        match self {
            Self::SourceTest => 1,
            Self::GoldenFixture => 2,
            Self::DifferentialExecution => 3,
            Self::DeclaredInvariant => 4,
            Self::TypeSignature => 5,
            Self::HumanApprovedSynthesized => 6,
        }
    }

    /// Whether this oracle alone is sufficient grounding for an `expected` value. A bare type
    /// signature never is — "a type signature alone is not sufficient evidence of expected
    /// behavior" (master prompt §11). Every other oracle kind carries some executed, declared, or
    /// human-reviewed evidence.
    pub fn is_grounded(&self) -> bool {
        !matches!(self, Self::TypeSignature)
    }
}

impl std::fmt::Display for OracleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceTest => write!(f, "source_test"),
            Self::GoldenFixture => write!(f, "golden_fixture"),
            Self::DifferentialExecution => write!(f, "differential_execution"),
            Self::DeclaredInvariant => write!(f, "declared_invariant"),
            Self::TypeSignature => write!(f, "type_signature"),
            Self::HumanApprovedSynthesized => write!(f, "human_approved_synthesized"),
        }
    }
}

/// Status of a behavioral contract's verification, mirroring `schemas/behavioral-contract.schema.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Passed,
    Failed,
    Pending,
    Degraded,
    Blocked,
}

impl std::fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Passed => write!(f, "passed"),
            Self::Failed => write!(f, "failed"),
            Self::Pending => write!(f, "pending"),
            Self::Degraded => write!(f, "degraded"),
            Self::Blocked => write!(f, "blocked"),
        }
    }
}

/// Behavioral assertion for unit contracts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehavioralAssertion {
    pub case_id: String,
    pub input: String,
    pub expected: String,
    pub oracle: OracleType,
    /// Where the evidence for `expected` came from (a repo path/URI, a captured differential-run
    /// artifact path, or an approver identity for human-approved cases). Never empty.
    pub evidence: String,
}

/// Behavioral contract defining expected unit behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehavioralContract {
    pub schema_version: String,
    pub contract_id: String,
    pub unit_id: String,
    pub unit_name: String,
    pub unit_kind: String,
    pub source_signature: String,
    pub target_signature: String,
    pub assertions: Vec<BehavioralAssertion>,
    pub verification_status: VerificationStatus,
}

impl BehavioralContract {
    /// A contract is grounded only if every assertion carries grounded evidence — one ungrounded
    /// (type-signature-only) assertion is enough to disqualify the whole contract from being
    /// counted toward grounded-oracle-coverage metrics.
    pub fn is_grounded(&self) -> bool {
        !self.assertions.is_empty() && self.assertions.iter().all(|a| a.oracle.is_grounded())
    }
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
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Generic error: {0}")]
    Generic(String),
    #[error("Other error: {0}")]
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
