//! Host toolchain inspection, discovery, and capability pre-flight checks for Project Exodus.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

/// Specific host tool classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolKind {
    Git,
    Cargo,
    Rustc,
    Python,
    Node,
    Java,
    Kotlinc,
    Docker,
    Agy,
    ClaudeCode,
    Codex,
    Ollama,
}

impl ToolKind {
    pub fn binary_name(&self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::Cargo => "cargo",
            Self::Rustc => "rustc",
            Self::Python => "python3",
            Self::Node => "node",
            Self::Java => "java",
            Self::Kotlinc => "kotlinc",
            Self::Docker => "docker",
            Self::Agy => "agy",
            Self::ClaudeCode => "claude",
            Self::Codex => "codex",
            Self::Ollama => "ollama",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Git => "Git CLI",
            Self::Cargo => "Cargo Package Manager",
            Self::Rustc => "Rust Compiler (rustc)",
            Self::Python => "Python 3 Runtime",
            Self::Node => "Node.js Runtime",
            Self::Java => "Java Development Kit",
            Self::Kotlinc => "Kotlin Compiler",
            Self::Docker => "Docker Container Engine",
            Self::Agy => "Antigravity CLI (agy)",
            Self::ClaudeCode => "Claude Code CLI",
            Self::Codex => "OpenAI Codex CLI",
            Self::Ollama => "Ollama Local LLM",
        }
    }

    pub fn install_guidance(&self) -> &'static str {
        match self {
            Self::Git => "Install Git via your package manager: `sudo apt install git` or `brew install git`",
            Self::Cargo | Self::Rustc => "Install Rust toolchain via rustup: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`",
            Self::Python => "Install Python 3.11+: `sudo apt install python3 python3-pip` or `brew install python`",
            Self::Node => "Install Node.js 20+: `nvm install 22` or `sudo apt install nodejs`",
            Self::Java => "Install OpenJDK 17+: `sudo apt install openjdk-17-jdk` or `brew install openjdk@17`",
            Self::Kotlinc => "Install Kotlin compiler: `sudo apt install kotlin` or `sdk install kotlin`",
            Self::Docker => "Install Docker Engine: https://docs.docker.com/engine/install/",
            Self::Agy => "Install Antigravity CLI or set ANTIGRAVITY_API_KEY / GEMINI_API_KEY",
            Self::ClaudeCode => "Install Claude Code CLI (`npm i -g @anthropic-ai/claude-code`) or set ANTHROPIC_API_KEY",
            Self::Codex => "Install OpenAI CLI (`npm i -g @openai/codex`) or set OPENAI_API_KEY",
            Self::Ollama => "Install Ollama from https://ollama.ai and start with `ollama serve`",
        }
    }
}

/// Status of host tool availability and compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolStatus {
    Available,
    Missing,
    Incompatible(String),
}

/// Execution execution mode for verification tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    Native,
    ContainerFallback,
    Unavailable,
}

/// Inspected tool details and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub kind: ToolKind,
    pub name: String,
    pub executable_path: Option<PathBuf>,
    pub version: Option<String>,
    pub required_version: Option<String>,
    pub status: ToolStatus,
    pub execution_mode: ExecutionMode,
    pub install_guidance: Option<String>,
}

/// Comprehensive toolchain inspection report for trajectory logs and CLI diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainReport {
    pub timestamp: DateTime<Utc>,
    pub tools: Vec<ToolInfo>,
    pub all_required_present: bool,
}

/// Toolchain inspector probing the host environment without shell interpolation.
pub struct ToolchainInspector;

impl ToolchainInspector {
    /// Safe binary probe using direct subprocess execution without shell expansion.
    pub fn probe_tool(kind: ToolKind) -> ToolInfo {
        let bin = kind.binary_name();
        let path_opt = which_binary(bin);

        let (status, version, mode) = if let Some(ref path) = path_opt {
            let ver = probe_version(path, kind);
            (ToolStatus::Available, ver, ExecutionMode::Native)
        } else {
            // Check if Docker fallback is available when a native tool is missing
            let docker_present = which_binary("docker").is_some();
            let mode = if docker_present && kind != ToolKind::Docker && kind != ToolKind::Git {
                ExecutionMode::ContainerFallback
            } else {
                ExecutionMode::Unavailable
            };
            (ToolStatus::Missing, None, mode)
        };

        ToolInfo {
            kind,
            name: kind.display_name().to_string(),
            executable_path: path_opt,
            version,
            required_version: None,
            status: status.clone(),
            execution_mode: mode,
            install_guidance: if status == ToolStatus::Missing {
                Some(kind.install_guidance().to_string())
            } else {
                None
            },
        }
    }

    /// Audits all standard host tools.
    pub fn audit_all() -> ToolchainReport {
        let kinds = [
            ToolKind::Git,
            ToolKind::Cargo,
            ToolKind::Rustc,
            ToolKind::Python,
            ToolKind::Node,
            ToolKind::Java,
            ToolKind::Kotlinc,
            ToolKind::Docker,
        ];

        let mut tools = Vec::new();
        let mut all_required = true;

        for kind in kinds {
            let info = Self::probe_tool(kind);
            if (kind == ToolKind::Git || kind == ToolKind::Cargo || kind == ToolKind::Rustc || kind == ToolKind::Python)
                && info.status != ToolStatus::Available
            {
                all_required = false;
            }
            tools.push(info);
        }

        ToolchainReport {
            timestamp: Utc::now(),
            tools,
            all_required_present: all_required,
        }
    }

    /// Audits specific source -> target migration toolchain requirements.
    pub fn audit_lane(source: &str, target: &str) -> ToolchainReport {
        let mut required_kinds = vec![ToolKind::Git];

        match source.to_lowercase().as_str() {
            "python" | "py" => required_kinds.push(ToolKind::Python),
            "javascript" | "js" | "typescript" | "ts" => required_kinds.push(ToolKind::Node),
            "java" => required_kinds.push(ToolKind::Java),
            "kotlin" | "kt" => required_kinds.push(ToolKind::Kotlinc),
            _ => {}
        }

        match target.to_lowercase().as_str() {
            "rust" | "rs" => {
                required_kinds.push(ToolKind::Rustc);
                required_kinds.push(ToolKind::Cargo);
            }
            "typescript" | "ts" | "javascript" | "js" => required_kinds.push(ToolKind::Node),
            "kotlin" | "kt" => required_kinds.push(ToolKind::Kotlinc),
            _ => {}
        }

        let mut tools = Vec::new();
        let mut all_required = true;

        for kind in required_kinds {
            let info = Self::probe_tool(kind);
            if info.status != ToolStatus::Available {
                all_required = false;
            }
            tools.push(info);
        }

        ToolchainReport {
            timestamp: Utc::now(),
            tools,
            all_required_present: all_required,
        }
    }

    /// Audits available host AI agent CLIs (Antigravity agy, Claude Code, OpenAI Codex, Ollama).
    pub fn detect_installed_agents() -> Vec<ToolInfo> {
        let agent_kinds = [
            ToolKind::Agy,
            ToolKind::ClaudeCode,
            ToolKind::Codex,
            ToolKind::Ollama,
        ];

        agent_kinds.into_iter().map(Self::probe_tool).collect()
    }
}

/// Safely discovers binary path using PATH lookup without shell interpolation.
fn which_binary(binary: &str) -> Option<PathBuf> {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(binary);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Safely probes tool version with explicit argument array.
fn probe_version(path: &PathBuf, kind: ToolKind) -> Option<String> {
    let arg = match kind {
        ToolKind::Java => "-version",
        _ => "--version",
    };

    let output = Command::new(path).arg(arg).output().ok()?;
    let text = if !output.stdout.is_empty() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    text.lines().next().map(|l| l.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_all_probes_cleanly() {
        let report = ToolchainInspector::audit_all();
        assert!(!report.tools.is_empty());
        let git_info = report.tools.iter().find(|t| t.kind == ToolKind::Git);
        assert!(git_info.is_some());
    }

    #[test]
    fn test_audit_lane_python_to_rust() {
        let report = ToolchainInspector::audit_lane("python", "rust");
        let kinds: Vec<ToolKind> = report.tools.iter().map(|t| t.kind).collect();
        assert!(kinds.contains(&ToolKind::Git));
        assert!(kinds.contains(&ToolKind::Python));
        assert!(kinds.contains(&ToolKind::Rustc));
        assert!(kinds.contains(&ToolKind::Cargo));
    }

    #[test]
    fn test_detect_installed_agents() {
        let agents = ToolchainInspector::detect_installed_agents();
        assert_eq!(agents.len(), 4);
        let names: Vec<&str> = agents.iter().map(|a| a.kind.binary_name()).collect();
        assert!(names.contains(&"agy"));
        assert!(names.contains(&"claude"));
        assert!(names.contains(&"codex"));
        assert!(names.contains(&"ollama"));
    }
}
