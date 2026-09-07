//! Axum HTTP Web Server and REST API for the Exodus Universal HITL Gate.
//!
//! Exposes an embedded local control plane and human-in-the-loop review dashboard.

pub mod assets;

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use exodus_core::{CiFailureEventPayload, DomainPayload, OperationalItem, SdlcIntegrationSettings};
use exodus_store::{EmbeddedOperationalStore, OperationalStore};
use exodus_verifier::MultiDomainVerifier;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::result::Result as StdResult;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

/// Shared application state across HTTP routes.
#[derive(Clone)]
pub struct WebState {
    pub store: Arc<Mutex<EmbeddedOperationalStore>>,
}

impl WebState {
    pub fn new(store: EmbeddedOperationalStore) -> Self {
        Self {
            store: Arc::new(Mutex::new(store)),
        }
    }
}

/// Request body for manual event ingestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestItemRequest {
    pub title: String,
    pub description: String,
    pub requester: String,
    pub payload: DomainPayload,
}

/// Request body for human approval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveRequest {
    pub approver: String,
    pub notes: String,
}

/// Request body for rejection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectRequest {
    pub actor: String,
    pub reason: String,
}

/// Builds the Axum router for Project Exodus HITL Web UI.
pub fn create_router(state: WebState) -> Router {
    Router::new()
        // Static UI pages
        .route("/", get(serve_index))
        .route("/static/style.css", get(serve_css))
        .route("/static/app.js", get(serve_js))
        // System endpoints
        .route("/api/health", get(health_check))
        .route("/api/operations", get(list_operations))
        .route("/api/operations/ingest", post(ingest_operation))
        .route("/api/operations/:id", get(get_operation))
        .route("/api/operations/:id/sandbox", post(sandbox_operation))
        .route("/api/operations/:id/verify", post(verify_operation))
        .route("/api/operations/:id/approve", post(approve_operation))
        .route("/api/operations/:id/reject", post(reject_operation))
        .route("/api/operations/:id/promote", post(promote_operation))
        // SDLC and Pipeline endpoints
        .route(
            "/api/sdlc/settings",
            get(get_sdlc_settings).post(save_sdlc_settings),
        )
        .route("/api/sdlc/scaffold", get(get_sdlc_scaffold))
        .route("/api/sdlc/webhook", post(handle_ci_webhook))
        .route("/api/graph", get(get_graph_topology))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Starts the embedded Axum HTTP server.
pub async fn start_server(
    mut store: EmbeddedOperationalStore,
    host: &str,
    port: u16,
) -> anyhow::Result<()> {
    let _ = store.seed_demo_fabric().await;
    let state = WebState::new(store);
    let app = create_router(state);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    println!("==================================================================");
    println!("  PROJECT EXODUS — Enterprise Agentic Execution Fabric");
    println!("  Universal Human-In-The-Loop Gate & Review UI Active");
    println!("  Listening on: http://{}", addr);
    println!("==================================================================");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// --- Handler Implementations ---

async fn serve_index() -> Html<&'static str> {
    Html(assets::INDEX_HTML)
}

async fn serve_css() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css")], assets::STYLE_CSS)
}

async fn serve_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        assets::APP_JS,
    )
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "Project Exodus Enterprise Agentic Execution Fabric",
        "engine": "SurrealDB Embedded Living Memory",
        "architecture": "6-Stage Closed Loop HITL Control Plane"
    }))
}

async fn list_operations(State(state): State<WebState>) -> Json<Vec<OperationalItem>> {
    let store = state.store.lock().await;
    let items = store.list_operational_items().await.unwrap_or_default();
    Json(items)
}

async fn get_operation(
    Path(id): Path<String>,
    State(state): State<WebState>,
) -> StdResult<Json<OperationalItem>, StatusCode> {
    let store = state.store.lock().await;
    match store.get_operational_item(&id).await {
        Ok(Some(item)) => Ok(Json(item)),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

async fn ingest_operation(
    State(state): State<WebState>,
    Json(req): Json<IngestItemRequest>,
) -> StdResult<Json<OperationalItem>, (StatusCode, String)> {
    let mut store = state.store.lock().await;
    let item = OperationalItem::new(req.title, req.description, req.requester, req.payload);
    store
        .save_operational_item(&item)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(item))
}

async fn sandbox_operation(
    Path(id): Path<String>,
    State(state): State<WebState>,
) -> StdResult<Json<OperationalItem>, (StatusCode, String)> {
    let mut store = state.store.lock().await;
    let mut item = store
        .get_operational_item(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Item not found".to_string()))?;

    let sandbox_path = format!(".exodus/worktrees/{}", item.id);
    item.mark_sandboxed("hitl-ui-operator", &sandbox_path)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    store
        .save_operational_item(&item)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(item))
}

async fn verify_operation(
    Path(id): Path<String>,
    State(state): State<WebState>,
) -> StdResult<Json<OperationalItem>, (StatusCode, String)> {
    let mut store = state.store.lock().await;
    let mut item = store
        .get_operational_item(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Item not found".to_string()))?;

    // Auto-advance to sandboxed if captured
    if item.state == exodus_core::OperationalLifecycleState::Captured {
        item.mark_sandboxed(
            "auto-sandbox-verifier",
            &format!(".exodus/worktrees/{}", item.id),
        )
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    }

    let cloned_store = store.clone();
    MultiDomainVerifier::verify_item(&mut item, &cloned_store, None)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    store
        .save_operational_item(&item)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(item))
}

async fn approve_operation(
    Path(id): Path<String>,
    State(state): State<WebState>,
    Json(req): Json<ApproveRequest>,
) -> StdResult<Json<OperationalItem>, (StatusCode, String)> {
    let mut store = state.store.lock().await;
    let mut item = store
        .get_operational_item(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Item not found".to_string()))?;

    item.approve(&req.approver, &req.notes)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    // Immediately promote to permanent organizational memory and generate regression fixtures
    if let Err(_e) =
        exodus_case::OperationalCasePromoter::promote(&mut item, std::path::Path::new(".exodus"))
    {
        let case_id = format!("CASE-{}", uuid::Uuid::now_v7());
        item.promote(&req.approver, &case_id)
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    }

    store
        .save_operational_item(&item)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(item))
}

async fn reject_operation(
    Path(id): Path<String>,
    State(state): State<WebState>,
    Json(req): Json<RejectRequest>,
) -> StdResult<Json<OperationalItem>, (StatusCode, String)> {
    let mut store = state.store.lock().await;
    let mut item = store
        .get_operational_item(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Item not found".to_string()))?;

    item.reject(&req.actor, &req.reason)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    store
        .save_operational_item(&item)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(item))
}

async fn promote_operation(
    Path(id): Path<String>,
    State(state): State<WebState>,
) -> StdResult<Json<OperationalItem>, (StatusCode, String)> {
    let mut store = state.store.lock().await;
    let mut item = store
        .get_operational_item(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Item not found".to_string()))?;

    let case_id = format!("CASE-{}", uuid::Uuid::now_v7());
    item.promote("hitl-operator", &case_id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    store
        .save_operational_item(&item)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(item))
}

async fn get_sdlc_settings(State(state): State<WebState>) -> Json<SdlcIntegrationSettings> {
    let store = state.store.lock().await;
    let settings = store.get_sdlc_settings().await.unwrap_or_default();
    Json(settings)
}

async fn save_sdlc_settings(
    State(state): State<WebState>,
    Json(settings): Json<SdlcIntegrationSettings>,
) -> StdResult<Json<SdlcIntegrationSettings>, (StatusCode, String)> {
    let mut store = state.store.lock().await;
    store
        .save_sdlc_settings(&settings)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(settings))
}

async fn get_sdlc_scaffold(State(state): State<WebState>) -> Json<HashMap<String, String>> {
    let store = state.store.lock().await;
    let scaffold = store
        .generate_pipeline_plugin_scaffold()
        .await
        .unwrap_or_default();
    Json(scaffold)
}

async fn handle_ci_webhook(
    State(state): State<WebState>,
    Json(payload): Json<CiFailureEventPayload>,
) -> StdResult<Json<OperationalItem>, (StatusCode, String)> {
    let mut store = state.store.lock().await;
    let item = store
        .ingest_ci_failure_event(payload)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(item))
}

async fn get_graph_topology() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "nodes": [
            {
                "id": "node_trigger",
                "label": "CI Failure #412",
                "kind": "trigger",
                "wave": 1,
                "status": "active",
                "subtitle": "Trigger: GitHub Actions CI Crash",
                "detail": "Pipeline failed in auth_integration suite. TokenVerifier panicked on unhandled ExpiredSignature error.",
                "diff_snippet": null,
                "meta": { "source": "GitHub Actions", "event": "CI Build #412", "branch": "master" }
            },
            {
                "id": "node_prompt",
                "label": "Developer Prompt",
                "kind": "prompt",
                "wave": 2,
                "status": "completed",
                "subtitle": "Prompt: Handle Token Expiration",
                "detail": "Prompt: 'In crates/auth/src/token.rs, safely catch ExpiredSignature and return AuthError::TokenExpired instead of panicking. Run all unit tests to confirm the fix.'",
                "diff_snippet": null,
                "meta": { "author": "engineer@exodus.dev", "model": "Claude 3.5 Sonnet" }
            },
            {
                "id": "node_action",
                "label": "AI Repair Task",
                "kind": "action",
                "wave": 3,
                "status": "completed",
                "subtitle": "Task: Safe Token Validation",
                "detail": "Generated bounded repair for token validation. Isolated changes inside git worktree sandbox and prepared regression tests.",
                "diff_snippet": null,
                "meta": { "strategy": "Bounded AST Repair", "sandbox": ".exodus/worktrees/op-eng-412" }
            },
            {
                "id": "node_code_file",
                "label": "auth/src/token.rs",
                "kind": "file_change",
                "wave": 4,
                "status": "modified",
                "subtitle": "Codebase File (+8, -2 lines)",
                "detail": "Modified authenticate_session to parse JWT claims safely and map expiration to structured error.",
                "diff_snippet": "@@ -40,7 +40,11 @@ fn authenticate_session(token: &str) -> Result<Session, AuthError> {\n-    let claims = parse_jwt_unchecked(token)?; // Panic on expired token\n+    let claims = match parse_jwt_safe(token) {\n+        Ok(c) => c,\n+        Err(JwtError::ExpiredSignature) => return Err(AuthError::TokenExpired),\n+        Err(e) => return Err(AuthError::InvalidToken(e.to_string())),\n+    };\n     validate_expiration(&claims)?;\n     Ok(Session::from_claims(claims))",
                "meta": { "file": "crates/auth/src/token.rs", "diff": "+8 / -2 lines" }
            },
            {
                "id": "node_fn_symbol",
                "label": "verify_token()",
                "kind": "symbol",
                "wave": 4,
                "status": "verified",
                "subtitle": "Function: AuthHandler::verify_token",
                "detail": "Exported public signature: pub async fn verify_token(&self, token: &str) -> Result<Session, AuthError>",
                "diff_snippet": null,
                "meta": { "visibility": "pub", "type": "async fn" }
            },
            {
                "id": "node_test_run",
                "label": "Automated Tests",
                "kind": "test",
                "wave": 5,
                "status": "passed",
                "subtitle": "cargo test (3 passed)",
                "detail": "Running 3 tests in crates/auth/tests/auth_integration.rs:\ntest test_valid_token ... ok\ntest test_token_expiration ... ok\ntest test_malformed_token ... ok\n\ntest result: ok. 3 passed; 0 failed; 0 ignored; finished in 0.42s",
                "diff_snippet": null,
                "meta": { "command": "cargo test --test auth_integration", "passed": "3", "failed": "0" }
            },
            {
                "id": "node_target_merge",
                "label": "Release Target",
                "kind": "target",
                "wave": 6,
                "status": "ready",
                "subtitle": "Ready for 1-Click Merge",
                "detail": "Clean diff with all automated tests passing. Ready for human review sign-off and branch promotion.",
                "diff_snippet": null,
                "meta": { "target_branch": "master", "status": "Ready for Review" }
            }
        ],
        "edges": [
            { "from": "node_trigger", "to": "node_prompt", "edge_type": "triggers", "is_cycle_edge": false },
            { "from": "node_prompt", "to": "node_action", "edge_type": "instructs", "is_cycle_edge": false },
            { "from": "node_action", "to": "node_code_file", "edge_type": "modifies", "is_cycle_edge": false },
            { "from": "node_code_file", "to": "node_fn_symbol", "edge_type": "contains", "is_cycle_edge": false },
            { "from": "node_code_file", "to": "node_test_run", "edge_type": "verified_by", "is_cycle_edge": false },
            { "from": "node_test_run", "to": "node_target_merge", "edge_type": "promotes_to", "is_cycle_edge": false }
        ],
        "total_cycles_detected": 0,
        "total_waves": 6,
        "grounded_oracle_count": 3,
        "title": "Change Lineage Graph",
        "description": "Trace how triggers and developer prompts lead directly to codebase file changes and automated test runs."
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use exodus_core::{CrmRequestPayload, DomainPayload};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_web_hitl_routes() {
        let store = EmbeddedOperationalStore::new();
        let state = WebState::new(store);
        let app = create_router(state);

        // 1. Check index.html route
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // 2. Check health route
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // 3. Ingest item via API
        let payload = DomainPayload::CrmRequest(CrmRequestPayload::new(
            "acc-api-test",
            "API Corp",
            150_000.0,
            15.0,
            "Enterprise",
            "AccountExec",
        ));
        let ingest_body = serde_json::to_string(&IngestItemRequest {
            title: "API Ingested Item".to_string(),
            description: "Testing API flow".to_string(),
            requester: "test-runner".to_string(),
            payload,
        })
        .unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/operations/ingest")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(ingest_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
