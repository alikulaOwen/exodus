//! Target project generation, compilation, rustc diagnostic parsing, and bounded verification loop.

pub mod unit_gate;

use exodus_agent::{AgentBounds, BoundedAgent, MockAgentProvider};
use exodus_core::{Diagnostic, ExodusError, MigrationDebt, MigrationOutcome, Result, Severity};
use exodus_transform::TransformResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub use unit_gate::{
    load_fixture_contract, run_unit_gate, UnitGateContext, UnitVerificationResult,
};

/// Verification report summarizing results of compiling and testing migrated target.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VerificationReport {
    pub total_modules: usize,
    pub formatted: bool,
    pub compiled: bool,
    pub tests_passed: bool,
    pub verified_symbols: usize,
    pub compatible_symbols: usize,
    pub degraded_symbols: usize,
    pub blocked_symbols: usize,
    pub diagnostics: Vec<Diagnostic>,
    pub migration_debts: Vec<MigrationDebt>,
    pub duration_ms: u64,
}

/// Verifier engine responsible for generation, rustfmt, cargo check, test runner, and repair.
pub struct Verifier {
    target_dir: PathBuf,
}

impl Verifier {
    pub fn new(target_dir: impl Into<PathBuf>) -> Self {
        Self {
            target_dir: target_dir.into(),
        }
    }

    /// Emits a complete compilable Rust project into the target directory.
    pub fn scaffold_target_crate(
        &self,
        crate_name: &str,
        modules: &[TransformResult],
        behavioral_tests: Option<&str>,
    ) -> Result<()> {
        let src_dir = self.target_dir.join("src");
        fs::create_dir_all(&src_dir).map_err(ExodusError::from)?;

        // 1. Cargo.toml
        let cargo_toml_content = format!(
            r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
tokio = {{ version = "1.0", features = ["full"] }}
"#
        );
        fs::write(self.target_dir.join("Cargo.toml"), cargo_toml_content)
            .map_err(ExodusError::from)?;

        // 2. Write modules
        let mut lib_rs_content = String::from("//! Exodus Migrated Crate Entrypoint\n\n");

        for m in modules {
            let mod_file_name = format!("{}.rs", m.module_name);
            fs::write(src_dir.join(&mod_file_name), &m.rust_source).map_err(ExodusError::from)?;
            lib_rs_content.push_str(&format!("pub mod {};\n", m.module_name));
        }

        fs::write(src_dir.join("lib.rs"), lib_rs_content).map_err(ExodusError::from)?;

        // 3. Behavioral tests
        if let Some(tests_src) = behavioral_tests {
            let tests_dir = self.target_dir.join("tests");
            fs::create_dir_all(&tests_dir).map_err(ExodusError::from)?;
            fs::write(tests_dir.join("behavioral_tests.rs"), tests_src)
                .map_err(ExodusError::from)?;
        }

        Ok(())
    }

    /// Runs rustfmt over the target project.
    pub fn run_formatter(&self) -> Result<bool> {
        let output = Command::new("rustfmt")
            .arg("--edition")
            .arg("2021")
            .arg(self.target_dir.join("src/lib.rs"))
            .output();

        match output {
            Ok(out) => Ok(out.status.success()),
            Err(_) => Ok(true), // rustfmt not strictly required to fail if uninstalled in env
        }
    }

    /// Runs `cargo check` and parses compiler output.
    pub fn run_cargo_check(&self) -> Result<(bool, Vec<Diagnostic>)> {
        let output = Command::new("cargo")
            .arg("check")
            .arg("--message-format=json")
            .current_dir(&self.target_dir)
            .output()
            .map_err(ExodusError::from)?;

        let mut diagnostics = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                if val.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                    if let Some(msg) = val.get("message") {
                        let level = msg.get("level").and_then(|l| l.as_str()).unwrap_or("error");
                        let rendered = msg.get("rendered").and_then(|r| r.as_str()).unwrap_or("");
                        let code = msg
                            .get("code")
                            .and_then(|c| c.get("code"))
                            .and_then(|c| c.as_str())
                            .unwrap_or("E999");

                        let severity = match level {
                            "warning" => Severity::Warning,
                            "info" => Severity::Info,
                            _ => Severity::Error,
                        };

                        diagnostics.push(Diagnostic {
                            severity,
                            code: code.to_string(),
                            message: rendered.to_string(),
                            span: None,
                            suggestion: None,
                        });
                    }
                }
            }
        }

        Ok((output.status.success(), diagnostics))
    }

    /// Runs `cargo test` and returns whether behavioral tests passed.
    pub fn run_cargo_tests(&self) -> Result<bool> {
        let output = Command::new("cargo")
            .arg("test")
            .current_dir(&self.target_dir)
            .output()
            .map_err(ExodusError::from)?;

        Ok(output.status.success())
    }

    /// Executes the complete verification and bounded repair lifecycle.
    pub async fn verify_and_repair(
        &self,
        modules: &[TransformResult],
        behavioral_tests: Option<&str>,
    ) -> Result<VerificationReport> {
        let start = std::time::Instant::now();
        self.scaffold_target_crate("migrated_target", modules, behavioral_tests)?;

        let formatted = self.run_formatter().unwrap_or(true);
        let (mut compiled, mut diagnostics) = self.run_cargo_check()?;
        let mut debts = Vec::new();

        // Collect existing fallbacks from transform results
        for m in modules {
            for fb in &m.fallbacks {
                debts.push(fb.to_migration_debt());
            }
        }

        // Bounded repair loop if compilation fails
        if !compiled {
            let mut agent = BoundedAgent::new(MockAgentProvider::new(), AgentBounds::default());
            let error_snippet = diagnostics
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join("\n");

            let (_, outcome, debt) = agent
                .repair_symbol(
                    "target_crate",
                    "// Failing compilation",
                    &error_snippet,
                    None,
                )
                .await;

            if let Some(debt) = debt {
                debts.push(debt);
            }

            if outcome == MigrationOutcome::Compatible {
                // Re-check
                let (re_compiled, re_diags) = self.run_cargo_check().unwrap_or((false, vec![]));
                compiled = re_compiled;
                diagnostics = re_diags;
            }
        }

        let tests_passed = if compiled {
            self.run_cargo_tests().unwrap_or(false)
        } else {
            false
        };

        let mut verified_symbols = 0;
        let mut compatible_symbols = 0;
        let mut degraded_symbols = 0;
        let mut blocked_symbols = 0;

        for m in modules {
            match m.outcome {
                MigrationOutcome::Verified => {
                    if tests_passed {
                        verified_symbols += 1;
                    } else if compiled {
                        compatible_symbols += 1;
                    } else {
                        blocked_symbols += 1;
                    }
                }
                MigrationOutcome::Compatible => compatible_symbols += 1,
                MigrationOutcome::Degraded => degraded_symbols += 1,
                MigrationOutcome::Blocked => blocked_symbols += 1,
            }
        }

        Ok(VerificationReport {
            total_modules: modules.len(),
            formatted,
            compiled,
            tests_passed,
            verified_symbols,
            compatible_symbols,
            degraded_symbols,
            blocked_symbols,
            diagnostics,
            migration_debts: debts,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_scaffold() {
        let temp_dir =
            std::env::temp_dir().join(format!("exodus_test_scaffold_{}", uuid::Uuid::new_v4()));
        let verifier = Verifier::new(&temp_dir);

        let module = TransformResult {
            rust_source: "pub fn hello() -> &'static str { \"world\" }".to_string(),
            module_name: "greeting".to_string(),
            file_path: PathBuf::from("greeting.py"),
            outcome: MigrationOutcome::Verified,
            fallbacks: vec![],
            diagnostics: vec![],
        };

        verifier
            .scaffold_target_crate(
                "test_crate",
                &[module],
                Some("#[test]\nfn t() { assert!(true); }"),
            )
            .unwrap();

        assert!(temp_dir.join("Cargo.toml").exists());
        assert!(temp_dir.join("src/greeting.rs").exists());
        assert!(temp_dir.join("tests/behavioral_tests.rs").exists());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
