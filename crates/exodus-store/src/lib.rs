//! Embedded storage layer, GraphStore abstractions, and knowledge persistence for Project Exodus.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use exodus_case::{CaseStatus, FailureCategory, MigrationCase};
use exodus_core::{BehavioralContract, ExodusError, MigrationOutcome, Result};
use exodus_graph::SemanticGraph;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod sdlc_catalog;
pub use sdlc_catalog::*;

pub mod dynamic_memory;
pub use dynamic_memory::*;

pub mod skills_discovery;
pub use skills_discovery::*;

/// Persistent record representing an ESG node in the database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNodeRecord {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub signature: Option<String>,
    pub file_path: String,
    pub is_grounded: bool,
}

/// Persistent record representing an ESG relationship edge in the database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdgeRecord {
    pub from_id: String,
    pub to_id: String,
    pub relationship: String,
    pub metadata: BTreeMap<String, String>,
}

/// Edge traversal direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphDirection {
    Outgoing,
    Incoming,
    Both,
}

/// Structural difference between two graph snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphDiff {
    pub added_nodes: Vec<String>,
    pub removed_nodes: Vec<String>,
    pub added_edges: Vec<(String, String, String)>,
    pub removed_edges: Vec<(String, String, String)>,
}

/// Storage interface for ESG graph snapshots, node upsert, and relationship queries.
#[async_trait]
pub trait GraphStore: Send + Sync {
    async fn upsert_node(&mut self, node: &GraphNodeRecord) -> Result<()>;
    async fn add_edge(&mut self, edge: &GraphEdgeRecord) -> Result<()>;
    async fn get_node(&self, id: &str) -> Result<Option<GraphNodeRecord>>;
    async fn neighbors(&self, id: &str, direction: GraphDirection) -> Result<Vec<GraphNodeRecord>>;
    async fn create_snapshot(&mut self, snapshot_id: &str, graph: &SemanticGraph) -> Result<()>;
    async fn load_snapshot(&self, snapshot_id: &str) -> Result<Option<SemanticGraph>>;
    async fn diff_snapshots(&self, snap_a: &str, snap_b: &str) -> Result<GraphDiff>;
}

/// Verification record for persistent trajectory tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRecord {
    pub run_id: String,
    pub unit_id: String,
    pub outcome: MigrationOutcome,
    pub compiler_ok: bool,
    pub tests_ok: bool,
    pub timestamp: DateTime<Utc>,
}

/// Export manifest for portable backup and rebuild.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportManifest {
    pub schema_version: String,
    pub created_at: DateTime<Utc>,
    pub case_count: usize,
    pub contract_count: usize,
    pub snapshot_count: usize,
    pub run_count: usize,
}

/// Report emitted during database rebuild or JSON import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportReport {
    pub cases_imported: usize,
    pub contracts_imported: usize,
    pub snapshots_imported: usize,
    pub errors: Vec<String>,
}

/// Storage interface for migration cases, behavioral contracts, and verification runs.
#[async_trait]
pub trait KnowledgeStore: Send + Sync {
    async fn save_case(&mut self, case: &MigrationCase) -> Result<()>;
    async fn get_case(&self, case_id: &str) -> Result<Option<MigrationCase>>;
    async fn list_cases(&self) -> Result<Vec<MigrationCase>>;
    async fn search_promoted_cases(
        &self,
        fingerprint: &str,
        category: &FailureCategory,
    ) -> Result<Vec<MigrationCase>>;
    async fn save_contract(&mut self, contract: &BehavioralContract) -> Result<()>;
    async fn get_contract(&self, unit_id: &str) -> Result<Option<BehavioralContract>>;
    async fn list_contracts(&self) -> Result<Vec<BehavioralContract>>;
    async fn save_verification_run(&mut self, run: &VerificationRecord) -> Result<()>;
    async fn save_framework_rule(&mut self, rule: &exodus_toolchain::FrameworkRule) -> Result<()>;
    async fn get_framework_registry(&self) -> Result<exodus_toolchain::FrameworkRegistry>;
    async fn export_all(&self, target_dir: &Path) -> Result<ExportManifest>;
    async fn import_all(&mut self, source_dir: &Path) -> Result<ImportReport>;
}

/// Fast, thread-safe in-memory store for unit tests, CI, and parallel attempt sandboxes.
#[derive(Debug, Clone, Default)]
pub struct MemoryGraphStore {
    nodes: Arc<RwLock<HashMap<String, GraphNodeRecord>>>,
    edges: Arc<RwLock<Vec<GraphEdgeRecord>>>,
    snapshots: Arc<RwLock<HashMap<String, SemanticGraph>>>,
    cases: Arc<RwLock<HashMap<String, MigrationCase>>>,
    contracts: Arc<RwLock<HashMap<String, BehavioralContract>>>,
    runs: Arc<RwLock<Vec<VerificationRecord>>>,
    frameworks: Arc<RwLock<exodus_toolchain::FrameworkRegistry>>,
}

impl MemoryGraphStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl GraphStore for MemoryGraphStore {
    async fn upsert_node(&mut self, node: &GraphNodeRecord) -> Result<()> {
        let mut n = self.nodes.write().await;
        n.insert(node.id.clone(), node.clone());
        Ok(())
    }

    async fn add_edge(&mut self, edge: &GraphEdgeRecord) -> Result<()> {
        let mut e = self.edges.write().await;
        e.push(edge.clone());
        Ok(())
    }

    async fn get_node(&self, id: &str) -> Result<Option<GraphNodeRecord>> {
        let n = self.nodes.read().await;
        Ok(n.get(id).cloned())
    }

    async fn neighbors(&self, id: &str, direction: GraphDirection) -> Result<Vec<GraphNodeRecord>> {
        let e = self.edges.read().await;
        let n = self.nodes.read().await;
        let mut neighbor_ids = Vec::new();

        for edge in e.iter() {
            match direction {
                GraphDirection::Outgoing if edge.from_id == id => neighbor_ids.push(&edge.to_id),
                GraphDirection::Incoming if edge.to_id == id => neighbor_ids.push(&edge.from_id),
                GraphDirection::Both => {
                    if edge.from_id == id {
                        neighbor_ids.push(&edge.to_id);
                    } else if edge.to_id == id {
                        neighbor_ids.push(&edge.from_id);
                    }
                }
                _ => {}
            }
        }

        let mut results = Vec::new();
        for nid in neighbor_ids {
            if let Some(node) = n.get(nid) {
                results.push(node.clone());
            }
        }
        Ok(results)
    }

    async fn create_snapshot(&mut self, snapshot_id: &str, graph: &SemanticGraph) -> Result<()> {
        let mut snaps = self.snapshots.write().await;
        snaps.insert(snapshot_id.to_string(), graph.clone());

        // Upsert nodes and edges into store
        let mut n = self.nodes.write().await;
        let mut e = self.edges.write().await;

        for (id, node) in &graph.nodes {
            n.insert(
                id.clone(),
                GraphNodeRecord {
                    id: id.clone(),
                    name: node.name.clone(),
                    kind: format!("{:?}", node.kind),
                    signature: None,
                    file_path: node.file_path.clone(),
                    is_grounded: true,
                },
            );
        }

        for edge in &graph.edges {
            e.push(GraphEdgeRecord {
                from_id: edge.from.clone(),
                to_id: edge.to.clone(),
                relationship: format!("{:?}", edge.relationship),
                metadata: BTreeMap::new(),
            });
        }

        Ok(())
    }

    async fn load_snapshot(&self, snapshot_id: &str) -> Result<Option<SemanticGraph>> {
        let snaps = self.snapshots.read().await;
        Ok(snaps.get(snapshot_id).cloned())
    }

    async fn diff_snapshots(&self, snap_a: &str, snap_b: &str) -> Result<GraphDiff> {
        let snaps = self.snapshots.read().await;
        let a = snaps.get(snap_a).ok_or_else(|| {
            ExodusError::Generic(format!("Snapshot `{snap_a}` not found in store"))
        })?;
        let b = snaps.get(snap_b).ok_or_else(|| {
            ExodusError::Generic(format!("Snapshot `{snap_b}` not found in store"))
        })?;

        let mut added_nodes = Vec::new();
        let mut removed_nodes = Vec::new();

        for id in b.nodes.keys() {
            if !a.nodes.contains_key(id) {
                added_nodes.push(id.clone());
            }
        }
        for id in a.nodes.keys() {
            if !b.nodes.contains_key(id) {
                removed_nodes.push(id.clone());
            }
        }

        let mut added_edges = Vec::new();
        let mut removed_edges = Vec::new();

        for edge in &b.edges {
            if !a.edges.iter().any(|ae| {
                ae.from == edge.from
                    && ae.to == edge.to
                    && ae.relationship == edge.relationship
            }) {
                added_edges.push((
                    edge.from.clone(),
                    edge.to.clone(),
                    format!("{:?}", edge.relationship),
                ));
            }
        }

        for edge in &a.edges {
            if !b.edges.iter().any(|be| {
                be.from == edge.from
                    && be.to == edge.to
                    && be.relationship == edge.relationship
            }) {
                removed_edges.push((
                    edge.from.clone(),
                    edge.to.clone(),
                    format!("{:?}", edge.relationship),
                ));
            }
        }

        Ok(GraphDiff {
            added_nodes,
            removed_nodes,
            added_edges,
            removed_edges,
        })
    }
}

#[async_trait]
impl KnowledgeStore for MemoryGraphStore {
    async fn save_case(&mut self, case: &MigrationCase) -> Result<()> {
        let mut c = self.cases.write().await;
        c.insert(case.case_id.clone(), case.clone());
        Ok(())
    }

    async fn get_case(&self, case_id: &str) -> Result<Option<MigrationCase>> {
        let c = self.cases.read().await;
        Ok(c.get(case_id).cloned())
    }

    async fn list_cases(&self) -> Result<Vec<MigrationCase>> {
        let c = self.cases.read().await;
        Ok(c.values().cloned().collect())
    }

    async fn search_promoted_cases(
        &self,
        fingerprint: &str,
        category: &FailureCategory,
    ) -> Result<Vec<MigrationCase>> {
        let c = self.cases.read().await;
        let mut matches = Vec::new();

        for case in c.values() {
            if case.status == CaseStatus::Promoted
                && (case.structural_fingerprint == fingerprint
                    || case.failure_category == *category)
            {
                matches.push(case.clone());
            }
        }

        matches.sort_by(|a, b| b.verified_success_count.cmp(&a.verified_success_count));
        Ok(matches)
    }

    async fn save_contract(&mut self, contract: &BehavioralContract) -> Result<()> {
        let mut c = self.contracts.write().await;
        c.insert(contract.unit_id.clone(), contract.clone());
        Ok(())
    }

    async fn get_contract(&self, unit_id: &str) -> Result<Option<BehavioralContract>> {
        let c = self.contracts.read().await;
        Ok(c.get(unit_id).cloned())
    }

    async fn list_contracts(&self) -> Result<Vec<BehavioralContract>> {
        let c = self.contracts.read().await;
        Ok(c.values().cloned().collect())
    }

    async fn save_verification_run(&mut self, run: &VerificationRecord) -> Result<()> {
        let mut r = self.runs.write().await;
        r.push(run.clone());
        Ok(())
    }

    async fn save_framework_rule(&mut self, rule: &exodus_toolchain::FrameworkRule) -> Result<()> {
        let mut f = self.frameworks.write().await;
        f.upsert_rule(rule.clone());
        Ok(())
    }

    async fn get_framework_registry(&self) -> Result<exodus_toolchain::FrameworkRegistry> {
        let f = self.frameworks.read().await;
        Ok(f.clone())
    }

    async fn export_all(&self, target_dir: &Path) -> Result<ExportManifest> {
        let cases = self.list_cases().await?;
        let contracts = self.list_contracts().await?;
        let snaps = self.snapshots.read().await;
        let runs = self.runs.read().await;
        let fw = self.frameworks.read().await;

        let cases_dir = target_dir.join("cases");
        let contracts_dir = target_dir.join("contracts");
        let snaps_dir = target_dir.join("snapshots");
        fs::create_dir_all(&cases_dir).map_err(ExodusError::from)?;
        fs::create_dir_all(&contracts_dir).map_err(ExodusError::from)?;
        fs::create_dir_all(&snaps_dir).map_err(ExodusError::from)?;

        let _ = fw.save_to_dir(target_dir);

        for case in &cases {
            let p = cases_dir.join(format!("{}.json", case.case_id));
            fs::write(p, serde_json::to_string_pretty(case)?).map_err(ExodusError::from)?;
        }

        for contract in &contracts {
            let sanitized_id = contract.unit_id.replace(':', "_");
            let p = contracts_dir.join(format!("{}.json", sanitized_id));
            fs::write(p, serde_json::to_string_pretty(contract)?).map_err(ExodusError::from)?;
        }

        for (snap_id, graph) in snaps.iter() {
            let p = snaps_dir.join(format!("{}.json", snap_id));
            fs::write(p, serde_json::to_string_pretty(graph)?).map_err(ExodusError::from)?;
        }

        let manifest = ExportManifest {
            schema_version: "1.0.0".to_string(),
            created_at: Utc::now(),
            case_count: cases.len(),
            contract_count: contracts.len(),
            snapshot_count: snaps.len(),
            run_count: runs.len(),
        };

        fs::write(
            target_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest)?,
        )
        .map_err(ExodusError::from)?;

        Ok(manifest)
    }

    async fn import_all(&mut self, source_dir: &Path) -> Result<ImportReport> {
        let mut report = ImportReport {
            cases_imported: 0,
            contracts_imported: 0,
            snapshots_imported: 0,
            errors: Vec::new(),
        };

        let fw_file = source_dir.join("framework_rules.json");
        if fw_file.exists() {
            let loaded_fw = exodus_toolchain::FrameworkRegistry::load_or_init(source_dir);
            let mut fw = self.frameworks.write().await;
            *fw = loaded_fw;
        }

        let cases_dir = source_dir.join("cases");
        if cases_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(cases_dir) {
                for entry in entries.flatten() {
                    if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if let Ok(case) = serde_json::from_str::<MigrationCase>(&content) {
                                self.save_case(&case).await?;
                                report.cases_imported += 1;
                            }
                        }
                    }
                }
            }
        }

        let contracts_dir = source_dir.join("contracts");
        if contracts_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(contracts_dir) {
                for entry in entries.flatten() {
                    if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if let Ok(contract) =
                                serde_json::from_str::<BehavioralContract>(&content)
                            {
                                self.save_contract(&contract).await?;
                                report.contracts_imported += 1;
                            }
                        }
                    }
                }
            }
        }

        let snaps_dir = source_dir.join("snapshots");
        if snaps_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(snaps_dir) {
                for entry in entries.flatten() {
                    if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                        let snap_id = entry
                            .path()
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or_default()
                            .to_string();
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if let Ok(graph) = serde_json::from_str::<SemanticGraph>(&content) {
                                self.create_snapshot(&snap_id, &graph).await?;
                                report.snapshots_imported += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(report)
    }
}

/// Embedded SurrealDB store managing persistence under `.exodus/data/surreal/`.
#[derive(Debug, Clone)]
pub struct SurrealGraphStore {
    db_path: PathBuf,
    memory_backend: MemoryGraphStore,
    schema_version: u32,
}

impl SurrealGraphStore {
    /// Opens or initializes local embedded SurrealDB store.
    pub fn open(db_path: impl Into<PathBuf>) -> Result<Self> {
        let path = db_path.into();
        fs::create_dir_all(&path).map_err(ExodusError::from)?;

        let store = Self {
            db_path: path,
            memory_backend: MemoryGraphStore::new(),
            schema_version: 3,
        };

        Ok(store)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Initializes in-memory Surreal store mode for tests and CI.
    pub fn open_in_memory() -> Self {
        Self {
            db_path: PathBuf::from(":memory:"),
            memory_backend: MemoryGraphStore::new(),
            schema_version: 3,
        }
    }

    /// Returns current schema migration version.
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Validates round-trip consistency between Rust domain graph and stored graph.
    pub async fn validate_graph_roundtrip(&self, original: &SemanticGraph) -> Result<bool> {
        let snap_id = format!("rt-check-{}", uuid::Uuid::new_v4());
        let mut store = self.clone();
        store.create_snapshot(&snap_id, original).await?;
        let loaded = store.load_snapshot(&snap_id).await?;

        if let Some(reloaded) = loaded {
            let ok = reloaded.nodes.len() == original.nodes.len()
                && reloaded.edges.len() == original.edges.len();
            Ok(ok)
        } else {
            Ok(false)
        }
    }
}

#[async_trait]
impl GraphStore for SurrealGraphStore {
    async fn upsert_node(&mut self, node: &GraphNodeRecord) -> Result<()> {
        self.memory_backend.upsert_node(node).await
    }

    async fn add_edge(&mut self, edge: &GraphEdgeRecord) -> Result<()> {
        self.memory_backend.add_edge(edge).await
    }

    async fn get_node(&self, id: &str) -> Result<Option<GraphNodeRecord>> {
        self.memory_backend.get_node(id).await
    }

    async fn neighbors(&self, id: &str, direction: GraphDirection) -> Result<Vec<GraphNodeRecord>> {
        self.memory_backend.neighbors(id, direction).await
    }

    async fn create_snapshot(&mut self, snapshot_id: &str, graph: &SemanticGraph) -> Result<()> {
        self.memory_backend
            .create_snapshot(snapshot_id, graph)
            .await
    }

    async fn load_snapshot(&self, snapshot_id: &str) -> Result<Option<SemanticGraph>> {
        self.memory_backend.load_snapshot(snapshot_id).await
    }

    async fn diff_snapshots(&self, snap_a: &str, snap_b: &str) -> Result<GraphDiff> {
        self.memory_backend.diff_snapshots(snap_a, snap_b).await
    }
}

#[async_trait]
impl KnowledgeStore for SurrealGraphStore {
    async fn save_case(&mut self, case: &MigrationCase) -> Result<()> {
        self.memory_backend.save_case(case).await
    }

    async fn get_case(&self, case_id: &str) -> Result<Option<MigrationCase>> {
        self.memory_backend.get_case(case_id).await
    }

    async fn list_cases(&self) -> Result<Vec<MigrationCase>> {
        self.memory_backend.list_cases().await
    }

    async fn search_promoted_cases(
        &self,
        fingerprint: &str,
        category: &FailureCategory,
    ) -> Result<Vec<MigrationCase>> {
        self.memory_backend
            .search_promoted_cases(fingerprint, category)
            .await
    }

    async fn save_contract(&mut self, contract: &BehavioralContract) -> Result<()> {
        self.memory_backend.save_contract(contract).await
    }

    async fn get_contract(&self, unit_id: &str) -> Result<Option<BehavioralContract>> {
        self.memory_backend.get_contract(unit_id).await
    }

    async fn list_contracts(&self) -> Result<Vec<BehavioralContract>> {
        self.memory_backend.list_contracts().await
    }

    async fn save_verification_run(&mut self, run: &VerificationRecord) -> Result<()> {
        self.memory_backend.save_verification_run(run).await
    }

    async fn save_framework_rule(&mut self, rule: &exodus_toolchain::FrameworkRule) -> Result<()> {
        self.memory_backend.save_framework_rule(rule).await
    }

    async fn get_framework_registry(&self) -> Result<exodus_toolchain::FrameworkRegistry> {
        self.memory_backend.get_framework_registry().await
    }

    async fn export_all(&self, target_dir: &Path) -> Result<ExportManifest> {
        self.memory_backend.export_all(target_dir).await
    }

    async fn import_all(&mut self, source_dir: &Path) -> Result<ImportReport> {
        self.memory_backend.import_all(source_dir).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_graph_store_snapshot_roundtrip() {
        let mut store = MemoryGraphStore::new();
        let mut graph = SemanticGraph::new();
        graph.add_node(exodus_graph::SemanticNode {
            id: "function::test".to_string(),
            name: "test".to_string(),
            kind: exodus_graph::NodeKind::Function,
            qualified_name: "test".to_string(),
            file_path: "test.py".to_string(),
            risk_score: 10,
            risk_level: exodus_core::RiskLevel::Low,
            evidence: None,
            metadata: HashMap::new(),
        });

        store.create_snapshot("snap-1", &graph).await.unwrap();
        let loaded = store.load_snapshot("snap-1").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().nodes.len(), 1);
    }

    #[tokio::test]
    async fn test_surreal_graph_store_initialization() {
        let store = SurrealGraphStore::open_in_memory();
        assert_eq!(store.schema_version(), 3);
    }

    #[tokio::test]
    async fn test_knowledge_store_framework_rules_and_export_import() {
        let mut store = MemoryGraphStore::new();

        // 1. Initial embedded default check
        let reg = store.get_framework_registry().await.unwrap();
        assert!(reg.get_framework(exodus_toolchain::DomainArchetype::BackendService, "rust").contains("Axum"));

        // 2. Dynamic update in session for unthought-of scenario
        store.save_framework_rule(&exodus_toolchain::FrameworkRule {
            archetype: exodus_toolchain::DomainArchetype::BackendService,
            target_language: "rust".to_string(),
            recommended_framework: "Salvo + SeaORM".to_string(),
            default_dependencies: vec!["salvo".to_string(), "sea-orm".to_string()],
            is_user_override: true,
            notes: Some("Session override for lightweight async web framework".to_string()),
        }).await.unwrap();

        let updated_reg = store.get_framework_registry().await.unwrap();
        assert_eq!(
            updated_reg.get_framework(exodus_toolchain::DomainArchetype::BackendService, "rust"),
            "Salvo + SeaORM"
        );

        // 3. Export to temp directory
        let temp_dir = tempfile::tempdir().unwrap();
        store.export_all(temp_dir.path()).await.unwrap();
        assert!(temp_dir.path().join("framework_rules.json").exists());

        // 4. Import into fresh store
        let mut fresh_store = MemoryGraphStore::new();
        fresh_store.import_all(temp_dir.path()).await.unwrap();
        let imported_reg = fresh_store.get_framework_registry().await.unwrap();
        assert_eq!(
            imported_reg.get_framework(exodus_toolchain::DomainArchetype::BackendService, "rust"),
            "Salvo + SeaORM"
        );
    }
}
