//! Bounded agent provider abstraction, repair controller, and secret-safe trajectory logging.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use exodus_core::{MigrationDebt, MigrationOutcome, Result, SourceSpan};
use exodus_fallback::FallbackGenerator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bounded configuration for agent interactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBounds {
    pub max_repair_iterations: usize,
    pub timeout_seconds: u64,
    pub temperature: f32,
    pub token_budget: usize,
    pub enforce_secret_scrubbing: bool,
}

impl Default for AgentBounds {
    fn default() -> Self {
        Self {
            max_repair_iterations: 3,
            timeout_seconds: 30,
            temperature: 0.1,
            token_budget: 16_000,
            enforce_secret_scrubbing: true,
        }
    }
}

/// Specialized bounded agent roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    ArchitectureAgent,
    MigrationAgent,
    RepairAgent,
}

/// Structured response returned by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub raw_content: String,
    pub proposed_code: Option<String>,
    pub explanation: String,
    pub tokens_prompt: usize,
    pub tokens_completion: usize,
    pub duration_ms: u64,
}

/// A recorded trajectory step in `.exodus/runs/<run_id>/trajectory.jsonl`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryStep {
    pub step_index: usize,
    pub timestamp: DateTime<Utc>,
    pub role: AgentRole,
    pub prompt_summary: String,
    pub response_summary: String,
    pub tokens_used: usize,
    pub cost_estimate_usd: f64,
}

/// Provider interface for LLM backends (deterministic mock, OpenAI-compatible, etc.).
#[async_trait]
pub trait AgentProvider: Send + Sync {
    fn provider_name(&self) -> &'static str;
    async fn complete(&self, system_prompt: &str, user_prompt: &str) -> Result<AgentResponse>;
}

/// Deterministic Mock Provider for reproducible offline execution, tests, and CI.
pub struct MockAgentProvider {
    pub canned_responses: HashMap<String, String>,
}

impl MockAgentProvider {
    pub fn new() -> Self {
        Self {
            canned_responses: HashMap::new(),
        }
    }

    pub fn with_response(mut self, pattern: &str, response: &str) -> Self {
        self.canned_responses
            .insert(pattern.to_string(), response.to_string());
        self
    }
}

impl Default for MockAgentProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentProvider for MockAgentProvider {
    fn provider_name(&self) -> &'static str {
        "MockAgentProvider"
    }

    async fn complete(&self, _system_prompt: &str, user_prompt: &str) -> Result<AgentResponse> {
        let mut proposed = None;
        for (pattern, res) in &self.canned_responses {
            if user_prompt.contains(pattern) {
                proposed = Some(res.clone());
                break;
            }
        }

        let proposed_code = proposed.or_else(|| {
            if user_prompt.contains("E0425") || user_prompt.contains("cannot find value") {
                Some(
                    "// Auto-repaired by MockAgent\npub fn repaired() -> bool { true }\n"
                        .to_string(),
                )
            } else {
                Some("// Synthesized code by MockAgent\npub fn synthesized() {}\n".to_string())
            }
        });

        Ok(AgentResponse {
            raw_content: proposed_code.clone().unwrap_or_default(),
            proposed_code,
            explanation: "Mock deterministic generation completed.".to_string(),
            tokens_prompt: 150,
            tokens_completion: 80,
            duration_ms: 12,
        })
    }
}

/// Bounded agent controller managing the repair loop and invariants.
pub struct BoundedAgent<P: AgentProvider> {
    provider: P,
    bounds: AgentBounds,
    fallback_gen: FallbackGenerator,
    trajectory: Vec<TrajectoryStep>,
}

impl<P: AgentProvider> BoundedAgent<P> {
    pub fn new(provider: P, bounds: AgentBounds) -> Self {
        Self {
            provider,
            bounds,
            fallback_gen: FallbackGenerator::new(),
            trajectory: Vec::new(),
        }
    }

    /// Scrubs secrets (API keys, bearer tokens) from string before recording.
    pub fn scrub_secrets(text: &str) -> String {
        let mut out = text.to_string();
        for key in [
            "OPENAI_API_KEY",
            "ANTHROPIC_API_KEY",
            "SECRET",
            "PASSWORD",
            "BEARER",
        ] {
            if let Some(pos) = out.find(key) {
                let end = (pos + 40).min(out.len());
                out.replace_range(pos..end, "[REDACTED_SECRET]");
            }
        }
        out
    }

    /// Executes a bounded repair loop on failing code (at most 3 iterations).
    pub async fn repair_symbol(
        &mut self,
        symbol_id: &str,
        source_code: &str,
        compiler_error: &str,
        location: Option<SourceSpan>,
    ) -> (String, MigrationOutcome, Option<MigrationDebt>) {
        let mut current_code = source_code.to_string();
        let current_error = compiler_error.to_string();

        for attempt in 1..=self.bounds.max_repair_iterations {
            let prompt = format!(
                "Attempt {attempt}/{}: Repair Rust code for symbol `{symbol_id}`.\nFailing code:\n{}\nCompiler Error:\n{}",
                self.bounds.max_repair_iterations, current_code, current_error
            );

            let scrubbed_prompt = if self.bounds.enforce_secret_scrubbing {
                Self::scrub_secrets(&prompt)
            } else {
                prompt.clone()
            };

            let sys_prompt = "You are the Exodus Repair Agent. Fix Rust compilation errors while preserving behavioral contracts.";
            let start = Utc::now();

            match self.provider.complete(sys_prompt, &scrubbed_prompt).await {
                Ok(response) => {
                    let cost = (response.tokens_prompt as f64 * 0.0000015)
                        + (response.tokens_completion as f64 * 0.000002);
                    self.trajectory.push(TrajectoryStep {
                        step_index: self.trajectory.len() + 1,
                        timestamp: start,
                        role: AgentRole::RepairAgent,
                        prompt_summary: format!("Repair attempt {attempt} on `{symbol_id}`"),
                        response_summary: response.explanation.clone(),
                        tokens_used: response.tokens_prompt + response.tokens_completion,
                        cost_estimate_usd: cost,
                    });

                    if let Some(fixed_code) = response.proposed_code {
                        current_code = fixed_code;
                        // In a mock/real workflow, if repair succeeds:
                        if !current_code.contains("error") {
                            return (current_code, MigrationOutcome::Compatible, None);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Agent repair query failed on attempt {attempt}: {e}");
                }
            }
        }

        // Bounded repair exceeded: Emit explicit fallback stub and record migration debt
        let fallback = self.fallback_gen.generate_stub(
            symbol_id,
            "repair_fallback",
            "/* repaired fallback */",
            "Result<(), String>",
            &format!(
                "Bounded repair loop exceeded {} iterations: {}",
                self.bounds.max_repair_iterations, current_error
            ),
            location,
        );

        let debt = fallback.to_migration_debt();
        (
            fallback.generated_code,
            MigrationOutcome::Degraded,
            Some(debt),
        )
    }

    pub fn trajectory(&self) -> &[TrajectoryStep] {
        &self.trajectory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_agent_repair_loop_success() {
        let mock_provider = MockAgentProvider::new();
        let bounds = AgentBounds::default();
        let mut agent = BoundedAgent::new(mock_provider, bounds);

        let (_code, outcome, debt) = agent
            .repair_symbol(
                "add_mod",
                "pub fn add() {}",
                "E0425: cannot find value",
                None,
            )
            .await;

        assert_eq!(outcome, MigrationOutcome::Compatible);
        assert!(debt.is_none());
        assert!(!agent.trajectory().is_empty());
    }

    #[tokio::test]
    async fn test_secret_scrubbing() {
        let text = "Here is my OPENAI_API_KEY=sk-proj1234567890abcdef1234567890 in config";
        let scrubbed = BoundedAgent::<MockAgentProvider>::scrub_secrets(text);
        assert!(!scrubbed.contains("sk-proj"));
        assert!(scrubbed.contains("[REDACTED_SECRET]"));
    }
}
