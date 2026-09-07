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
