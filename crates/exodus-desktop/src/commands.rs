//! Desktop IPC Commands bridging the native Tauri window to Exodus engines.

use exodus_core::{
    DomainPayload, OperationalDomainTag, OperationalItem, OperationalLifecycleState,
    SdlcIntegrationSettings,
};
use exodus_store::{EmbeddedOperationalStore, OperationalStore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// Shared application state managed by Tauri.
pub struct DesktopState {
    pub store: Arc<Mutex<EmbeddedOperationalStore>>,
    pub storage_path: PathBuf,
}

impl DesktopState {
    pub async fn new() -> anyhow::Result<Self> {
        let storage_path = PathBuf::from(".exodus/fabric_store.json");
        let store = EmbeddedOperationalStore::load_or_init(&storage_path).await?;
        Ok(Self {
            store: Arc::new(Mutex::new(store)),
            storage_path,
        })
    }
}

/// Representation of a node in the change lineage graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsgGraphNode {
    pub id: String,
    pub label: String,
    pub kind: String, // "trigger", "prompt", "action", "file_change", "symbol", "test", "target"
    pub wave: usize,
    pub in_cycle: bool,
    pub status: String, // "active", "completed", "modified", "verified", "passed", "ready"
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub diff_snippet: Option<String>,
    #[serde(default)]
    pub meta: Option<HashMap<String, String>>,
}

/// Representation of a directed causal dependency edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsgGraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: String, // "triggers", "instructs", "modifies", "contains", "verified_by", "promotes_to"
    pub is_cycle_edge: bool,
}

/// Complete Change Graph Topology payload for the visualizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsgTopologyPayload {
    pub nodes: Vec<EsgGraphNode>,
    pub edges: Vec<EsgGraphEdge>,
    pub total_cycles_detected: usize,
    pub total_waves: usize,
    pub grounded_oracle_count: usize,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[tauri::command]
pub async fn list_operations(
    state: State<'_, DesktopState>,
) -> Result<Vec<OperationalItem>, String> {
    let store = state.store.lock().await;
    store
        .list_operational_items()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_operation(
    id: String,
    state: State<'_, DesktopState>,
) -> Result<Option<OperationalItem>, String> {
    let store = state.store.lock().await;
    store
        .get_operational_item(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn verify_operation(
    id: String,
    state: State<'_, DesktopState>,
) -> Result<OperationalItem, String> {
    let mut store = state.store.lock().await;
    let mut item = store
        .get_operational_item(&id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Operational item '{}' not found", id))?;

    if item.state == OperationalLifecycleState::Captured {
        item.mark_sandboxed(
            "tauri-desktop-operator",
            &format!(".exodus/worktrees/{}", item.id),
        )
        .map_err(|e| e.to_string())?;
    }

    let cloned_store = store.clone();
    exodus_verifier::MultiDomainVerifier::verify_item(&mut item, &cloned_store, None)
        .await
        .map_err(|e| e.to_string())?;

    store
        .save_operational_item(&item)
        .await
        .map_err(|e| e.to_string())?;
    let _ = store.persist_to_disk(&state.storage_path).await;

    Ok(item)
}

#[tauri::command]
pub async fn approve_operation(
    id: String,
    approver: String,
    notes: Option<String>,
    state: State<'_, DesktopState>,
) -> Result<OperationalItem, String> {
    let mut store = state.store.lock().await;
    let mut item = store
        .get_operational_item(&id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Operational item '{}' not found", id))?;

    item.approve(
        &approver,
        notes
            .as_deref()
            .unwrap_or("Approved via Exodus Mission Control Desktop"),
    )
    .map_err(|e| e.to_string())?;

    if let Err(_e) =
        exodus_case::OperationalCasePromoter::promote(&mut item, std::path::Path::new(".exodus"))
    {
        let case_id = format!("CASE-{}", uuid::Uuid::now_v7());
        item.promote(&approver, &case_id)
            .map_err(|e| e.to_string())?;
    }

    store
        .save_operational_item(&item)
        .await
        .map_err(|e| e.to_string())?;
    let _ = store.persist_to_disk(&state.storage_path).await;

    Ok(item)
}

#[tauri::command]
pub async fn reject_operation(
    id: String,
    actor: String,
    reason: String,
    state: State<'_, DesktopState>,
) -> Result<OperationalItem, String> {
    let mut store = state.store.lock().await;
    let mut item = store
        .get_operational_item(&id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Operational item '{}' not found", id))?;

    item.reject(&actor, &reason).map_err(|e| e.to_string())?;

    store
        .save_operational_item(&item)
        .await
        .map_err(|e| e.to_string())?;
    let _ = store.persist_to_disk(&state.storage_path).await;

    Ok(item)
}

#[tauri::command]
pub async fn ingest_operation(
    title: String,
    description: Option<String>,
    requester: String,
    domain_tag: String,
    payload: serde_json::Value,
    state: State<'_, DesktopState>,
) -> Result<OperationalItem, String> {
    let mut store = state.store.lock().await;
    let tag: OperationalDomainTag = domain_tag.parse().map_err(|e: String| e)?;

    let domain_payload: DomainPayload = serde_json::from_value(payload)
        .map_err(|e| format!("Invalid domain payload format: {}", e))?;

    let mut item = OperationalItem::new(
        title,
        description.unwrap_or_else(|| "Ingested via Exodus Desktop".to_string()),
        requester,
        domain_payload,
    );
    item.domain_tag = tag;

    store
        .save_operational_item(&item)
        .await
        .map_err(|e| e.to_string())?;
    let _ = store.persist_to_disk(&state.storage_path).await;

    Ok(item)
}

#[tauri::command]
pub async fn get_sdlc_settings(
    state: State<'_, DesktopState>,
) -> Result<SdlcIntegrationSettings, String> {
    let store = state.store.lock().await;
    store.get_sdlc_settings().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_sdlc_settings(
    settings: SdlcIntegrationSettings,
    state: State<'_, DesktopState>,
) -> Result<(), String> {
    let mut store = state.store.lock().await;
    store
        .save_sdlc_settings(&settings)
        .await
        .map_err(|e| e.to_string())?;
    let _ = store.persist_to_disk(&state.storage_path).await;
    Ok(())
}

#[tauri::command]
pub async fn get_sdlc_scaffold(
    state: State<'_, DesktopState>,
) -> Result<HashMap<String, String>, String> {
    let store = state.store.lock().await;
    store
        .generate_pipeline_plugin_scaffold()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_esg_topology(_sample_id: Option<String>) -> Result<EsgTopologyPayload, String> {
    // Grounded Change Lineage Graph linking Trigger -> Prompt -> Task Action -> Codebase Change -> Automated Tests -> Target
    let mut meta_trigger = HashMap::new();
    meta_trigger.insert("source".to_string(), "GitHub Actions".to_string());
    meta_trigger.insert("event".to_string(), "CI Build #412".to_string());
    meta_trigger.insert("branch".to_string(), "master".to_string());

    let mut meta_prompt = HashMap::new();
    meta_prompt.insert("author".to_string(), "engineer@exodus.dev".to_string());
    meta_prompt.insert("model".to_string(), "Claude 3.5 Sonnet".to_string());

    let mut meta_action = HashMap::new();
    meta_action.insert("strategy".to_string(), "Bounded AST Repair".to_string());
    meta_action.insert(
        "sandbox".to_string(),
        ".exodus/worktrees/op-eng-412".to_string(),
    );

    let mut meta_file = HashMap::new();
    meta_file.insert("file".to_string(), "crates/auth/src/token.rs".to_string());
    meta_file.insert("diff".to_string(), "+8 / -2 lines".to_string());

    let mut meta_test = HashMap::new();
    meta_test.insert(
        "command".to_string(),
        "cargo test --test auth_integration".to_string(),
    );
    meta_test.insert("passed".to_string(), "3".to_string());
    meta_test.insert("failed".to_string(), "0".to_string());

    let mut meta_target = HashMap::new();
    meta_target.insert("target_branch".to_string(), "master".to_string());
    meta_target.insert("status".to_string(), "Ready for Review".to_string());

    let nodes = vec![
        EsgGraphNode {
            id: "node_trigger".to_string(),
            label: "CI Failure #412".to_string(),
            kind: "trigger".to_string(),
            wave: 1,
            in_cycle: false,
            status: "active".to_string(),
            subtitle: Some("Trigger: GitHub Actions CI Crash".to_string()),
            detail: Some("Pipeline failed in auth_integration suite. TokenVerifier panicked on unhandled ExpiredSignature error.".to_string()),
            diff_snippet: None,
            meta: Some(meta_trigger),
        },
        EsgGraphNode {
            id: "node_prompt".to_string(),
            label: "Developer Prompt".to_string(),
            kind: "prompt".to_string(),
            wave: 2,
            in_cycle: false,
            status: "completed".to_string(),
            subtitle: Some("Prompt: Handle Token Expiration".to_string()),
            detail: Some("Prompt: 'In crates/auth/src/token.rs, safely catch ExpiredSignature and return AuthError::TokenExpired instead of panicking. Run all unit tests to confirm the fix.'".to_string()),
            diff_snippet: None,
            meta: Some(meta_prompt),
        },
        EsgGraphNode {
            id: "node_action".to_string(),
            label: "AI Repair Task".to_string(),
            kind: "action".to_string(),
            wave: 3,
            in_cycle: false,
            status: "completed".to_string(),
            subtitle: Some("Task: Safe Token Validation".to_string()),
            detail: Some("Generated bounded repair for token validation. Isolated changes inside git worktree sandbox and prepared regression tests.".to_string()),
            diff_snippet: None,
            meta: Some(meta_action),
        },
        EsgGraphNode {
            id: "node_code_file".to_string(),
            label: "auth/src/token.rs".to_string(),
            kind: "file_change".to_string(),
            wave: 4,
            in_cycle: false,
            status: "modified".to_string(),
            subtitle: Some("Codebase File (+8, -2 lines)".to_string()),
            detail: Some("Modified authenticate_session to parse JWT claims safely and map expiration to structured error.".to_string()),
            diff_snippet: Some("@@ -40,7 +40,11 @@ fn authenticate_session(token: &str) -> Result<Session, AuthError> {\n-    let claims = parse_jwt_unchecked(token)?; // Panic on expired token\n+    let claims = match parse_jwt_safe(token) {\n+        Ok(c) => c,\n+        Err(JwtError::ExpiredSignature) => return Err(AuthError::TokenExpired),\n+        Err(e) => return Err(AuthError::InvalidToken(e.to_string())),\n+    };\n     validate_expiration(&claims)?;\n     Ok(Session::from_claims(claims))".to_string()),
            meta: Some(meta_file),
        },
        EsgGraphNode {
            id: "node_fn_symbol".to_string(),
            label: "verify_token()".to_string(),
            kind: "symbol".to_string(),
            wave: 4,
            in_cycle: false,
            status: "verified".to_string(),
            subtitle: Some("Function: AuthHandler::verify_token".to_string()),
            detail: Some("Exported public signature: pub async fn verify_token(&self, token: &str) -> Result<Session, AuthError>".to_string()),
            diff_snippet: None,
            meta: None,
        },
        EsgGraphNode {
            id: "node_test_run".to_string(),
            label: "Automated Tests".to_string(),
            kind: "test".to_string(),
            wave: 5,
            in_cycle: false,
            status: "passed".to_string(),
            subtitle: Some("cargo test (3 passed)".to_string()),
            detail: Some("Running 3 tests in crates/auth/tests/auth_integration.rs:\ntest test_valid_token ... ok\ntest test_token_expiration ... ok\ntest test_malformed_token ... ok\n\ntest result: ok. 3 passed; 0 failed; 0 ignored; finished in 0.42s".to_string()),
            diff_snippet: None,
            meta: Some(meta_test),
        },
        EsgGraphNode {
            id: "node_target_merge".to_string(),
            label: "Release Target".to_string(),
            kind: "target".to_string(),
            wave: 6,
            in_cycle: false,
            status: "ready".to_string(),
            subtitle: Some("Ready for 1-Click Merge".to_string()),
            detail: Some("Clean diff with all automated tests passing. Ready for human review sign-off and branch promotion.".to_string()),
            diff_snippet: None,
            meta: Some(meta_target),
        },
    ];

    let edges = vec![
        EsgGraphEdge {
            from: "node_trigger".to_string(),
            to: "node_prompt".to_string(),
            edge_type: "triggers".to_string(),
            is_cycle_edge: false,
        },
        EsgGraphEdge {
            from: "node_prompt".to_string(),
            to: "node_action".to_string(),
            edge_type: "instructs".to_string(),
            is_cycle_edge: false,
        },
        EsgGraphEdge {
            from: "node_action".to_string(),
            to: "node_code_file".to_string(),
            edge_type: "modifies".to_string(),
            is_cycle_edge: false,
        },
        EsgGraphEdge {
            from: "node_code_file".to_string(),
            to: "node_fn_symbol".to_string(),
            edge_type: "contains".to_string(),
            is_cycle_edge: false,
        },
        EsgGraphEdge {
            from: "node_code_file".to_string(),
            to: "node_test_run".to_string(),
            edge_type: "verified_by".to_string(),
            is_cycle_edge: false,
        },
        EsgGraphEdge {
            from: "node_test_run".to_string(),
            to: "node_target_merge".to_string(),
            edge_type: "promotes_to".to_string(),
            is_cycle_edge: false,
        },
    ];

    Ok(EsgTopologyPayload {
        nodes,
        edges,
        total_cycles_detected: 0,
        total_waves: 6,
        grounded_oracle_count: 3,
        title: Some("Change Lineage Graph".to_string()),
        description: Some("Trace how triggers and developer prompts lead directly to codebase file changes and automated test runs.".to_string()),
    })
}
