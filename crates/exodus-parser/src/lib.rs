//! Parsing subsystem interfaces and Tree-sitter abstraction hooks for legacy codebases.

use exodus_core::{Language, Result};
use std::path::Path;

/// Trait defining a source code parser interface for a language frontend.
pub trait SourceParser {
    /// Returns the language supported by this parser instance.
    fn language(&self) -> Language;

    /// Parses a source file into a structured representation.
    fn parse_file(&self, path: &Path) -> Result<ParsedSource>;
}

/// Minimal representation of a parsed source file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedSource {
    pub file_path: String,
    pub language: Language,
    pub symbols: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsed_source_init() {
        let parsed = ParsedSource {
            file_path: "test.py".to_string(),
            language: Language::Python,
            symbols: vec!["foo".to_string()],
        };
        assert_eq!(parsed.language, Language::Python);
        assert_eq!(parsed.symbols.len(), 1);
    }
}
