//! Fallback handlers and migration debt emitters for unsupported semantics.

use exodus_core::{MigrationDebt, Result};
use serde::{Deserialize, Serialize};

/// Strategy for generating fallback stubs when a construct cannot be fully converted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackStrategy {
    TodoStub,
    PanicStub,
    InterfaceMock,
}

/// Fallback generator for unmapped semantics.
pub struct FallbackGenerator;

impl FallbackGenerator {
    pub fn generate_fallback(
        symbol_id: &str,
        strategy: FallbackStrategy,
        reason: &str,
    ) -> Result<(String, MigrationDebt)> {
        let code = match strategy {
            FallbackStrategy::TodoStub => format!("todo!(\"Exodus Migration Debt: {reason}\");\n"),
            FallbackStrategy::PanicStub => {
                format!("panic!(\"Unimplemented semantic construct: {reason}\");\n")
            }
            FallbackStrategy::InterfaceMock => {
                format!("/* Fallback Mock for {symbol_id}: {reason} */\n")
            }
        };

        let debt = MigrationDebt {
            symbol_id: symbol_id.to_string(),
            description: format!("Generated fallback for {symbol_id}"),
            reason: reason.to_string(),
            location: None,
        };

        Ok((code, debt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_fallback() {
        let (code, debt) = FallbackGenerator::generate_fallback(
            "fn_eval",
            FallbackStrategy::TodoStub,
            "eval() dynamic invocation",
        )
        .unwrap();
        assert!(code.contains("todo!"));
        assert_eq!(debt.symbol_id, "fn_eval");
    }
}
