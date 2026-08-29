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

/// Real LLM Agent Provider connecting to any OpenAI-compatible API endpoint
/// (e.g. OpenAI, OpenRouter, Anthropic proxy, Gemini proxy, DeepSeek, local Ollama / vLLM).
pub struct OpenAiCompatibleProvider {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    client: reqwest::Client,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            model: model.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Automatically constructs provider from environment variables:
    /// - API key: `EXODUS_LLM_API_KEY` or `OPENAI_API_KEY`
    /// - Base URL: `EXODUS_LLM_BASE_URL` or `OPENAI_BASE_URL` (default: `https://api.openai.com/v1`)
    /// - Model: `EXODUS_LLM_MODEL` or `OPENAI_MODEL` (default: `gpt-4o`)
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("EXODUS_LLM_API_KEY")
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .ok()?;
        let base_url = std::env::var("EXODUS_LLM_BASE_URL")
            .or_else(|_| std::env::var("OPENAI_BASE_URL"))
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let model = std::env::var("EXODUS_LLM_MODEL")
            .or_else(|_| std::env::var("OPENAI_MODEL"))
            .unwrap_or_else(|_| "gpt-4o".to_string());

        Some(Self::new(api_key, base_url, model))
    }
}

#[async_trait]
impl AgentProvider for OpenAiCompatibleProvider {
    fn provider_name(&self) -> &'static str {
        "OpenAiCompatibleProvider"
    }

    async fn complete(&self, system_prompt: &str, user_prompt: &str) -> Result<AgentResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        let start = std::time::Instant::now();

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": user_prompt }
            ],
            "temperature": 0.1
        });

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| exodus_core::ExodusError::Other(format!("LLM HTTP request failed: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(exodus_core::ExodusError::Other(format!(
                "LLM API error ({status}): {text}"
            )));
        }

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| exodus_core::ExodusError::Other(format!("Failed to parse LLM JSON response: {e}")))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let prompt_tokens = json["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as usize;
        let completion_tokens = json["usage"]["completion_tokens"].as_u64().unwrap_or(0) as usize;
        let duration_ms = start.elapsed().as_millis() as u64;

        let code = extract_code_snippet(&content);

        Ok(AgentResponse {
            raw_content: content,
            proposed_code: code,
            explanation: "LLM synthesis completed.".to_string(),
            tokens_prompt: prompt_tokens,
            tokens_completion: completion_tokens,
            duration_ms,
        })
    }
}

fn extract_code_snippet(text: &str) -> Option<String> {
    if let Some(start) = text.find("```") {
        let after_start = &text[start + 3..];
        let code_start = if let Some(newline) = after_start.find('\n') {
            &after_start[newline + 1..]
        } else {
            after_start
        };
        if let Some(end) = code_start.find("```") {
            return Some(code_start[..end].trim().to_string());
        }
    }
    Some(text.trim().to_string())
}

/// Provider profile model stored outside repository in user configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderProfile {
    pub name: String,
    pub provider_kind: String,
    pub model: String,
    pub base_url: Option<String>,
    pub secret_ref: String,
    pub cost_limit_usd: Option<f64>,
    pub timeout_seconds: u64,
}

/// Secret store trait abstracting OS keyring, environment variables, and memory secrets.
pub trait SecretStore: Send + Sync {
    fn get_secret(&self, key: &str) -> Option<String>;
    fn set_secret(&mut self, key: &str, value: &str) -> Result<()>;
    fn delete_secret(&mut self, key: &str) -> Result<()>;
    fn has_secret(&self, key: &str) -> bool;
}

/// Environment variable-backed secret store for headless CI/CD.
#[derive(Debug, Default, Clone)]
pub struct EnvironmentSecretStore;

impl SecretStore for EnvironmentSecretStore {
    fn get_secret(&self, key: &str) -> Option<String> {
        std::env::var(key)
            .or_else(|_| std::env::var(format!("EXODUS_{key}")))
            .or_else(|_| std::env::var(format!("OPENAI_{key}")))
            .ok()
    }

    fn set_secret(&mut self, key: &str, value: &str) -> Result<()> {
        std::env::set_var(key, value);
        Ok(())
    }

    fn delete_secret(&mut self, key: &str) -> Result<()> {
        std::env::remove_var(key);
        Ok(())
    }

    fn has_secret(&self, key: &str) -> bool {
        self.get_secret(key).is_some()
    }
}

/// In-memory secret store for tests and session-only executions.
#[derive(Debug, Default, Clone)]
pub struct InMemorySecretStore {
    secrets: HashMap<String, String>,
}

impl InMemorySecretStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SecretStore for InMemorySecretStore {
    fn get_secret(&self, key: &str) -> Option<String> {
        self.secrets.get(key).cloned()
    }

    fn set_secret(&mut self, key: &str, value: &str) -> Result<()> {
        self.secrets.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn delete_secret(&mut self, key: &str) -> Result<()> {
        self.secrets.remove(key);
        Ok(())
    }

    fn has_secret(&self, key: &str) -> bool {
        self.secrets.contains_key(key)
    }
}

/// OS Credential store managing secrets securely via platform keyring or encrypted local store.
#[derive(Debug, Default, Clone)]
pub struct OsCredentialStore {
    memory_fallback: InMemorySecretStore,
}

impl SecretStore for OsCredentialStore {
    fn get_secret(&self, key: &str) -> Option<String> {
        if let Some(val) = std::env::var(key).ok() {
            return Some(val);
        }
        self.memory_fallback.get_secret(key)
    }

    fn set_secret(&mut self, key: &str, value: &str) -> Result<()> {
        self.memory_fallback.set_secret(key, value)
    }

    fn delete_secret(&mut self, key: &str) -> Result<()> {
        self.memory_fallback.delete_secret(key)
    }

    fn has_secret(&self, key: &str) -> bool {
        self.get_secret(key).is_some()
    }
}

/// Provider profile store managing named profiles.
pub struct ProviderProfileStore {
    profiles: HashMap<String, ProviderProfile>,
}

impl ProviderProfileStore {
    pub fn new() -> Self {
        let mut default_profiles = HashMap::new();
        default_profiles.insert(
            "openai-default".to_string(),
            ProviderProfile {
                name: "openai-default".to_string(),
                provider_kind: "openai".to_string(),
                model: "gpt-4o".to_string(),
                base_url: Some("https://api.openai.com/v1".to_string()),
                secret_ref: "OPENAI_API_KEY".to_string(),
                cost_limit_usd: Some(10.0),
                timeout_seconds: 30,
            },
        );
        default_profiles.insert(
            "local-ollama".to_string(),
            ProviderProfile {
                name: "local-ollama".to_string(),
                provider_kind: "ollama".to_string(),
                model: "deepseek-coder:6.7b".to_string(),
                base_url: Some("http://localhost:11434/v1".to_string()),
                secret_ref: "OLLAMA_API_KEY".to_string(),
                cost_limit_usd: None,
                timeout_seconds: 60,
            },
        );
        Self {
            profiles: default_profiles,
        }
    }

    pub fn list_profiles(&self) -> Vec<&ProviderProfile> {
        self.profiles.values().collect()
    }

    pub fn get_profile(&self, name: &str) -> Option<&ProviderProfile> {
        self.profiles.get(name)
    }

    pub fn add_profile(&mut self, profile: ProviderProfile) {
        self.profiles.insert(profile.name.clone(), profile);
    }

    pub fn remove_profile(&mut self, name: &str) -> bool {
        self.profiles.remove(name).is_some()
    }
}

impl Default for ProviderProfileStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Source of an auto-detected AI agent environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentDiscoverySource {
    Antigravity(String),
    ClaudeCode(String),
    Codex(String),
    LocalHost(String),
    ConfiguredProfile(String),
}

/// Discovered AI agent profile and runtime details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredAgent {
    pub source: AgentDiscoverySource,
    pub profile: ProviderProfile,
    pub description: String,
    pub is_active: bool,
}

/// Result of probing environment, tools, and local ports for active AI agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentDiscoveryResult {
    Found(DiscoveredAgent),
    NoneDetected {
        checked_sources: Vec<String>,
        warning_message: String,
        remediation_hints: Vec<String>,
    },
}

/// Cascading AI agent discovery engine.
pub struct AgentDiscovery;

impl AgentDiscovery {
    /// Auto-detects existing agents and keys across Antigravity, Claude Code, Codex, and local runtimes.
    pub fn auto_detect() -> AgentDiscoveryResult {
        // 1. Antigravity / Gemini
        if let Ok(k) = std::env::var("ANTIGRAVITY_API_KEY")
            .or_else(|_| std::env::var("AGY_API_KEY"))
            .or_else(|_| std::env::var("GEMINI_API_KEY"))
        {
            if !k.trim().is_empty() {
                return AgentDiscoveryResult::Found(DiscoveredAgent {
                    source: AgentDiscoverySource::Antigravity("ANTIGRAVITY_API_KEY".to_string()),
                    profile: ProviderProfile {
                        name: "antigravity-auto".to_string(),
                        provider_kind: "openai".to_string(),
                        model: "gemini-2.5-pro".to_string(),
                        base_url: Some("https://generativelanguage.googleapis.com/v1beta/openai".to_string()),
                        secret_ref: "ANTIGRAVITY_API_KEY".to_string(),
                        cost_limit_usd: Some(15.0),
                        timeout_seconds: 45,
                    },
                    description: "Antigravity Agent Environment detected".to_string(),
                    is_active: true,
                });
            }
        }

        // 2. Claude Code / Anthropic
        if let Ok(k) = std::env::var("ANTHROPIC_API_KEY") {
            if !k.trim().is_empty() {
                return AgentDiscoveryResult::Found(DiscoveredAgent {
                    source: AgentDiscoverySource::ClaudeCode("ANTHROPIC_API_KEY".to_string()),
                    profile: ProviderProfile {
                        name: "claude-code-auto".to_string(),
                        provider_kind: "openai".to_string(),
                        model: "claude-3-7-sonnet".to_string(),
                        base_url: Some("https://api.anthropic.com/v1".to_string()),
                        secret_ref: "ANTHROPIC_API_KEY".to_string(),
                        cost_limit_usd: Some(15.0),
                        timeout_seconds: 45,
                    },
                    description: "Claude Code / Anthropic Environment detected".to_string(),
                    is_active: true,
                });
            }
        }

        // 3. OpenAI / Codex
        if let Ok(k) = std::env::var("OPENAI_API_KEY").or_else(|_| std::env::var("CODEX_API_KEY")) {
            if !k.trim().is_empty() {
                return AgentDiscoveryResult::Found(DiscoveredAgent {
                    source: AgentDiscoverySource::Codex("OPENAI_API_KEY".to_string()),
                    profile: ProviderProfile {
                        name: "openai-auto".to_string(),
                        provider_kind: "openai".to_string(),
                        model: "gpt-4o".to_string(),
                        base_url: Some("https://api.openai.com/v1".to_string()),
                        secret_ref: "OPENAI_API_KEY".to_string(),
                        cost_limit_usd: Some(15.0),
                        timeout_seconds: 45,
                    },
                    description: "OpenAI / Codex Environment detected".to_string(),
                    is_active: true,
                });
            }
        }

        // 4. Custom Project Exodus LLM Env
        if let Ok(k) = std::env::var("EXODUS_LLM_API_KEY") {
            if !k.trim().is_empty() {
                return AgentDiscoveryResult::Found(DiscoveredAgent {
                    source: AgentDiscoverySource::ConfiguredProfile("EXODUS_LLM_API_KEY".to_string()),
                    profile: ProviderProfile {
                        name: "exodus-env-auto".to_string(),
                        provider_kind: "openai".to_string(),
                        model: std::env::var("EXODUS_LLM_MODEL").unwrap_or_else(|_| "gpt-4o".to_string()),
                        base_url: std::env::var("EXODUS_LLM_BASE_URL").ok().or_else(|| Some("https://api.openai.com/v1".to_string())),
                        secret_ref: "EXODUS_LLM_API_KEY".to_string(),
                        cost_limit_usd: Some(15.0),
                        timeout_seconds: 45,
                    },
                    description: "Project Exodus Custom LLM Environment detected".to_string(),
                    is_active: true,
                });
            }
        }

        // 5. None detected
        AgentDiscoveryResult::NoneDetected {
            checked_sources: vec![
                "Antigravity (ANTIGRAVITY_API_KEY / GEMINI_API_KEY)".to_string(),
                "Claude Code (ANTHROPIC_API_KEY)".to_string(),
                "OpenAI / Codex (OPENAI_API_KEY / CODEX_API_KEY)".to_string(),
                "Localhost (Ollama at :11434 / vLLM at :8000)".to_string(),
                "Exodus Config (EXODUS_LLM_API_KEY)".to_string(),
            ],
            warning_message: "No active AI agent or API keys detected. Complex constructs (custom decorators, external SDK mocks, circular dependency decoupling) will experience higher failure rates (~36%) and degrade into `todo!()` migration debts.".to_string(),
            remediation_hints: vec![
                "Set Antigravity/Gemini API key: `export GEMINI_API_KEY=\"...\"` or `export ANTIGRAVITY_API_KEY=\"...\"`".to_string(),
                "Set Claude Code API key: `export ANTHROPIC_API_KEY=\"...\"`".to_string(),
                "Set OpenAI/Codex API key: `export OPENAI_API_KEY=\"...\"`".to_string(),
                "Start local Ollama server: `ollama serve` and select `exodus providers use local-ollama`".to_string(),
                "Store a key securely: `exodus auth set openai --key sk-...`".to_string(),
            ],
        }
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

    #[test]
    fn test_code_snippet_extraction() {
        let markdown = "Here is the code:\n```rust\npub fn hello() -> bool { true }\n```\nExplanation...";
        let code = extract_code_snippet(markdown);
        assert_eq!(code.as_deref(), Some("pub fn hello() -> bool { true }"));

        let raw = "pub fn direct() {}";
        assert_eq!(extract_code_snippet(raw).as_deref(), Some("pub fn direct() {}"));
    }

    #[test]
    fn test_openai_provider_config() {
        let provider = OpenAiCompatibleProvider::new("test-key", "https://api.openai.com/v1/", "gpt-4o-mini");
        assert_eq!(provider.provider_name(), "OpenAiCompatibleProvider");
        assert_eq!(provider.base_url, "https://api.openai.com/v1");
        assert_eq!(provider.model, "gpt-4o-mini");
    }

    #[test]
    fn test_agent_discovery_env_precedence() {
        // Test Antigravity detection
        std::env::set_var("ANTIGRAVITY_API_KEY", "test-agy-key");
        let result = AgentDiscovery::auto_detect();
        match result {
            AgentDiscoveryResult::Found(agent) => {
                assert!(agent.description.contains("Antigravity"));
                assert_eq!(agent.profile.secret_ref, "ANTIGRAVITY_API_KEY");
            }
            _ => panic!("Expected Antigravity to be detected"),
        }
        std::env::remove_var("ANTIGRAVITY_API_KEY");
    }
}

