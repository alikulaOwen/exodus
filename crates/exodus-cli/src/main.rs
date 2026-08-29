//! Exodus Command-Line Interface (CLI).

use clap::{Parser, Subcommand, ValueEnum};
use exodus_agent::AgentBounds;
use exodus_case::CaseEngine;
use exodus_eval::Evaluator;
use exodus_graph::SemanticGraph;
use exodus_parser::{PythonParser, SourceParser};
use exodus_planner::{MigrationPlan, MigrationPlanner};
use exodus_transform::TransformationEngine;
use exodus_verifier::{
    load_fixture_contract, run_gated_migration, run_unit_gate, unit_id_for, UnitGateContext,
    Verifier,
};
use exodus_worktree::WorktreeManager;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "exodus")]
#[command(about = "Project Exodus: Graph-guided, agent-assisted legacy code migration engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output results in JSON format
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a source repository and construct the Exodus Semantic Graph
    Analyze {
        /// Source directory path
        #[arg(default_value = ".")]
        source: PathBuf,

        /// Output directory for .exodus artifacts
        #[arg(short, long, default_value = ".exodus")]
        output: PathBuf,
    },
    /// Generate an ordered migration plan with risk weights and approval checkpoints
    Plan {
        /// Source directory path
        #[arg(default_value = ".")]
        source: PathBuf,

        /// Output directory for .exodus artifacts
        #[arg(short, long, default_value = ".exodus")]
        output: PathBuf,
    },
    /// Approve a generated migration plan to unlock migration
    Approve {
        /// Path to plan.json
        #[arg(default_value = ".exodus/plan.json")]
        plan: PathBuf,

        /// Approver identity / name
        #[arg(short, long, default_value = "human_architect")]
        approver: String,
    },
    /// Execute transformation and generate target Rust workspace
    Migrate {
        /// Source directory path
        #[arg(default_value = ".")]
        source: PathBuf,

        /// Target output directory for migrated Rust code
        #[arg(short, long, default_value = "target/exodus_migrated")]
        output: PathBuf,

        /// Bypass approval requirement (for automated pipelines)
        #[arg(long)]
        force: bool,

        /// Route through the per-unit verification gate: dependency-ordered unit-by-unit
        /// generate/compile/contract-verify inside an Exodus-owned worktree, with a real atomic
        /// commit per verified unit and a merge proposal at the end (master prompt §11/§15),
        /// instead of the single whole-repository transform+scaffold below.
        #[arg(long)]
        gated: bool,
    },
    /// Verify target Rust crate (format, check, test, bounded repair)
    Verify {
        /// Migrated target directory
        #[arg(default_value = "target/exodus_migrated")]
        target: PathBuf,
    },
    /// Display aggregate migration outcome report and debt summary
    Report {
        /// Directory containing .exodus artifacts
        #[arg(short, long, default_value = ".exodus")]
        dir: PathBuf,
    },
    /// Run full evaluation benchmark on synthetic fixtures
    Eval {
        /// Fixtures directory path
        #[arg(short, long, default_value = "fixtures")]
        fixtures: PathBuf,

        /// Output directory for evaluation artifacts
        #[arg(short, long, default_value = ".exodus")]
        output: PathBuf,
    },
    /// Manage migration units and boundary extraction
    #[command(subcommand)]
    Units(UnitsCommands),
    /// Manage unit-level behavioral contracts
    #[command(subcommand)]
    Contracts(ContractsCommands),
    /// Governed Migration Case Engine
    #[command(subcommand)]
    Cases(CasesCommands),
    /// Git worktree isolation and leasing
    #[command(subcommand)]
    Worktree(WorktreeCommands),
}

#[derive(Subcommand)]
enum UnitsCommands {
    /// List extracted migration units for a repository
    List {
        #[arg(default_value = ".")]
        source: PathBuf,
    },
    /// Show details for a specific migration unit
    Show {
        unit_id: String,
        #[arg(short, long, default_value = ".")]
        source: PathBuf,
    },
    /// Run isolated unit gate verification
    Verify {
        unit_id: String,
        #[arg(short, long, default_value = ".")]
        source: PathBuf,
    },
}

#[derive(Subcommand)]
enum ContractsCommands {
    /// Show behavioral contract for a unit
    Show {
        unit_id: String,
        #[arg(short, long, default_value = ".")]
        source: PathBuf,
    },
    /// Verify behavioral contract assertions
    Verify {
        unit_id: String,
        #[arg(short, long, default_value = ".")]
        source: PathBuf,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum TestMode {
    Replay,
    Verify,
}

#[derive(Subcommand)]
enum CasesCommands {
    /// List all captured and promoted migration cases
    List,
    /// Show details of a specific migration case
    Show { case_id: String },
    /// Search promoted cases structurally
    Search { query: String },
    /// Review candidate migration case
    Review { case_id: String },
    /// Approve and promote a verified migration case
    Approve {
        case_id: String,
        #[arg(short, long, default_value = "LeadArchitect")]
        approver: String,
    },
    /// Reject/deprecate a migration case
    Reject { case_id: String },
    /// Test promoted cases in replay or full verify mode
    Test {
        #[arg(short, long, value_enum, default_value_t = TestMode::Replay)]
        mode: TestMode,
    },
}

#[derive(Subcommand)]
enum WorktreeCommands {
    /// List active and recoverable migration worktree leases
    List,
    /// Inspect a specific worktree task
    Inspect { task_id: String },
    /// Resume an existing worktree lease
    Resume { task_id: String },
    /// Preserve a dirty/review-required worktree
    Preserve { task_id: String },
    /// Safely cleanup a worktree lease
    Cleanup { task_id: String },
    /// Force discard a worktree lease with confirmation
    Discard {
        task_id: String,
        #[arg(long)]
        confirm: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { source, output } => {
            fs::create_dir_all(&output)?;
            let parser = PythonParser::new();
            println!("🔍 Parsing repository at {}...", source.display());
            let parsed = parser.parse_repository(&source)?;
            println!("📊 Building Exodus Semantic Graph (ESG)...");
            let graph = SemanticGraph::from_parsed_repository(&parsed);
            let summary = graph.generate_architecture_summary();

            let graph_json = serde_json::to_string_pretty(&graph)?;
            let arch_json = serde_json::to_string_pretty(&summary)?;

            fs::write(output.join("graph.json"), &graph_json)?;
            fs::write(output.join("architecture.json"), &arch_json)?;

            if cli.json {
                println!("{}", arch_json);
            } else {
                println!("✅ ESG Analysis complete!");
                println!("   - Total Nodes: {}", summary.total_nodes);
                println!("   - Total Edges: {}", summary.total_edges);
                println!("   - Modules: {}", summary.module_count);
                println!("   - Types/Classes: {}", summary.type_count);
                println!("   - Functions: {}", summary.function_count);
                println!("   - Unsupported Constructs: {}", summary.unsupported_count);
                println!(
                    "   - Dependency Cycles: {}",
                    summary.circular_dependencies.len()
                );
                println!("📁 Artifacts written to {}", output.display());
            }
        }
        Commands::Plan { source, output } => {
            fs::create_dir_all(&output)?;
            let parser = PythonParser::new();
            let parsed = parser.parse_repository(&source)?;
            let graph = SemanticGraph::from_parsed_repository(&parsed);
            let planner = MigrationPlanner::new();
            let plan = planner.generate_plan(&graph)?;

            let plan_json = plan.to_json()?;
            fs::write(output.join("plan.json"), &plan_json)?;

            if cli.json {
                println!("{}", plan_json);
            } else {
                println!("📋 Migration Plan Generated: {}", plan.plan_id);
                println!("   - Total Steps: {}", plan.steps.len());
                println!("   - High Risk Symbols: {}", plan.high_risk_symbols.len());
                println!(
                    "   - Unsupported Constructs: {}",
                    plan.unsupported_constructs.len()
                );
                println!("   - Approval Required: {}", !plan.is_approved());
                if !plan.is_approved() {
                    println!("⚠️  Approval Checkpoints:");
                    for reason in &plan.approval.required_reasons {
                        println!("      • {reason}");
                    }
                    println!(
                        "\nRun `exodus approve` to approve this plan before running migration."
                    );
                } else {
                    println!("✅ Plan automatically pre-approved (0 risky blockers).");
                }
                println!("📁 Written to {}", output.join("plan.json").display());
            }
        }
        Commands::Approve { plan, approver } => {
            if !plan.exists() {
                eprintln!("❌ Plan file not found: {}", plan.display());
                std::process::exit(1);
            }
            let plan_str = fs::read_to_string(&plan)?;
            let mut plan_data = MigrationPlan::from_json(&plan_str)?;
            plan_data.approve(&approver);
            fs::write(&plan, plan_data.to_json()?)?;

            if cli.json {
                println!("{}", plan_data.to_json()?);
            } else {
                println!(
                    "✅ Migration plan approved by `{}` at {:?}",
                    approver, plan_data.approval.approval_timestamp
                );
            }
        }
        Commands::Migrate {
            source,
            output,
            force,
            gated,
        } => {
            let exodus_dir = Path::new(".exodus");
            let plan_path = exodus_dir.join("plan.json");

            if !force && plan_path.exists() {
                let plan_str = fs::read_to_string(&plan_path)?;
                let plan_data = MigrationPlan::from_json(&plan_str)?;
                if !plan_data.is_approved() {
                    eprintln!("❌ Migration blocked: Plan has not been approved yet. Run `exodus approve` or use `--force`.");
                    std::process::exit(1);
                }
            }

            if gated {
                let task_id = format!("cli-{}", Uuid::new_v4());
                let run_id = format!("run-{}", Uuid::new_v4());
                let repo_root = std::env::current_dir()?;
                println!(
                    "🚀 Running gated per-unit migration for {} (task `{task_id}`)...",
                    source.display()
                );
                let summary =
                    run_gated_migration(&source, &repo_root, Some(&source), &task_id, &run_id)
                        .await?;

                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&summary)?);
                } else {
                    println!("📊 Gated Migration Summary:");
                    println!(
                        "   - Units: {} verified, {} compatible, {} degraded, {} blocked",
                        summary.verified_unit_count,
                        summary.compatible_unit_count,
                        summary.degraded_unit_count,
                        summary.blocked_unit_count
                    );
                    println!(
                        "   - Grounded contracts: {}/{}",
                        summary.grounded_contract_count,
                        summary.unit_results.len()
                    );
                    println!(
                        "   - Module integration: {}",
                        if summary.module_integration_passed {
                            "✅ Passed"
                        } else {
                            "❌ Failed / no verified units"
                        }
                    );
                    println!("   - Atomic commits: {}", summary.commits.len());
                    if let Some(branch) = &summary.branch_name {
                        println!("   - Worktree branch: {branch}");
                    }
                    if let Some(path) = &summary.worktree_path {
                        println!("   - Worktree path: {}", path.display());
                    }
                    if let Some(proposal) = &summary.merge_proposal {
                        println!(
                            "   - Merge proposal: {} (approved: {})",
                            proposal.proposal_id, proposal.approved
                        );
                    }
                }
                return Ok(());
            }

            println!(
                "🚀 Executing migration transformations for {}...",
                source.display()
            );
            let parser = PythonParser::new();
            let parsed = parser.parse_repository(&source)?;
            let engine = TransformationEngine::new();
            let results = engine.transform_repository(&parsed)?;

            let mut all_fallbacks = Vec::new();
            for r in &results {
                all_fallbacks.extend(r.fallbacks.clone());
            }

            fs::create_dir_all(exodus_dir)?;
            fs::write(
                exodus_dir.join("fallbacks.json"),
                serde_json::to_string_pretty(&all_fallbacks)?,
            )?;

            let verifier = Verifier::new(&output);
            verifier.scaffold_target_crate("migrated_exodus_target", &results, None)?;

            if cli.json {
                let json_res = serde_json::to_string_pretty(&results)?;
                println!("{}", json_res);
            } else {
                println!("✅ Migration completed!");
                println!("   - Transformed Modules: {}", results.len());
                println!("   - Fallback Records (Debt): {}", all_fallbacks.len());
                println!("📁 Generated Rust workspace in: {}", output.display());
                println!(
                    "📁 Fallback ledger written to: {}",
                    exodus_dir.join("fallbacks.json").display()
                );
            }
        }
        Commands::Verify { target } => {
            println!("🧪 Verifying migrated target at {}...", target.display());
            let verifier = Verifier::new(&target);
            let (compiled, diags) = verifier.run_cargo_check()?;
            let formatted = verifier.run_formatter().unwrap_or(true);
            let tests_passed = if compiled {
                verifier.run_cargo_tests().unwrap_or(false)
            } else {
                false
            };

            let report = serde_json::json!({
                "target": target.display().to_string(),
                "formatted": formatted,
                "compiled": compiled,
                "tests_passed": tests_passed,
                "diagnostics_count": diags.len(),
            });

            fs::create_dir_all(".exodus")?;
            fs::write(
                ".exodus/report.json",
                serde_json::to_string_pretty(&report)?,
            )?;

            if cli.json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("📊 Verification Report:");
                println!(
                    "   - Formatting: {}",
                    if formatted {
                        "✅ Formatted"
                    } else {
                        "❌ Unformatted"
                    }
                );
                println!(
                    "   - Compilation: {}",
                    if compiled { "✅ Passed" } else { "❌ Failed" }
                );
                println!(
                    "   - Behavioral Tests: {}",
                    if tests_passed {
                        "✅ Passed"
                    } else {
                        "⚠️ Pending / None"
                    }
                );
                println!("   - Diagnostics: {}", diags.len());
                println!("📁 Written to .exodus/report.json");
            }
        }
        Commands::Report { dir } => {
            let report_path = dir.join("report.json");
            let fallbacks_path = dir.join("fallbacks.json");
            let arch_path = dir.join("architecture.json");
            let contracts_dir = dir.join("contracts");

            // Unit-level verification hierarchy: aggregated from real per-unit
            // `verification.json` artifacts (see exodus_verifier::unit_gate), reported as its own
            // tier — a repository must never be presented as "verified" merely because some units
            // passed, and unit successes must not be erased by a later module/repo-level failure.
            let mut unit_results: Vec<exodus_verifier::UnitVerificationResult> = Vec::new();
            if contracts_dir.exists() {
                if let Ok(entries) = fs::read_dir(&contracts_dir) {
                    for entry in entries.flatten() {
                        let vfile = entry.path().join("verification.json");
                        if let Ok(content) = fs::read_to_string(&vfile) {
                            if let Ok(v) = serde_json::from_str::<
                                exodus_verifier::UnitVerificationResult,
                            >(&content)
                            {
                                unit_results.push(v);
                            }
                        }
                    }
                }
            }

            let module_integration_status = if report_path.exists() {
                fs::read_to_string(&report_path)
                    .ok()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                    .and_then(|v| v.get("compiled").and_then(|c| c.as_bool()))
            } else {
                None
            };

            if cli.json {
                let unit_count = unit_results.len();
                let verified = unit_results
                    .iter()
                    .filter(|u| u.outcome == exodus_core::MigrationOutcome::Verified)
                    .count();
                let compatible = unit_results
                    .iter()
                    .filter(|u| u.outcome == exodus_core::MigrationOutcome::Compatible)
                    .count();
                let degraded = unit_results
                    .iter()
                    .filter(|u| u.outcome == exodus_core::MigrationOutcome::Degraded)
                    .count();
                let blocked = unit_results
                    .iter()
                    .filter(|u| u.outcome == exodus_core::MigrationOutcome::Blocked)
                    .count();
                let cluster_count = unit_results
                    .iter()
                    .filter(|u| u.boundary_kind == "cluster")
                    .count();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "verification_hierarchy": {
                            "unit": { "total": unit_count, "verified": verified, "compatible": compatible, "degraded": degraded, "blocked": blocked },
                            "dependency_cluster": { "total": cluster_count },
                            "module_integration": { "status": module_integration_status },
                        },
                        "units": unit_results,
                    }))?
                );
            } else {
                println!("📈 Project Exodus Status Report:\n");
                println!("=== Verification Hierarchy ===");
                if unit_results.is_empty() {
                    println!(
                        "Unit tier: no per-unit verification.json artifacts found under {}",
                        contracts_dir.display()
                    );
                } else {
                    let verified = unit_results
                        .iter()
                        .filter(|u| u.outcome == exodus_core::MigrationOutcome::Verified)
                        .count();
                    let compatible = unit_results
                        .iter()
                        .filter(|u| u.outcome == exodus_core::MigrationOutcome::Compatible)
                        .count();
                    let degraded = unit_results
                        .iter()
                        .filter(|u| u.outcome == exodus_core::MigrationOutcome::Degraded)
                        .count();
                    let blocked = unit_results
                        .iter()
                        .filter(|u| u.outcome == exodus_core::MigrationOutcome::Blocked)
                        .count();
                    let clusters = unit_results
                        .iter()
                        .filter(|u| u.boundary_kind == "cluster")
                        .count();
                    println!(
                        "Unit tier: {} evaluated ({verified} verified, {compatible} compatible, {degraded} degraded, {blocked} blocked); {clusters} were dependency clusters.",
                        unit_results.len()
                    );
                }
                match module_integration_status {
                    Some(true) => {
                        println!("Module integration tier: ✅ compiled (from report.json)")
                    }
                    Some(false) => {
                        println!("Module integration tier: ❌ did not compile (from report.json)")
                    }
                    None => println!("Module integration tier: no report.json found — not yet run"),
                }
                println!(
                    "Repository/end-to-end tier: not separately tracked by this command — unit and module-integration success above never implies this.\n"
                );
            }

            if !cli.json {
                if report_path.exists() {
                    let rep = fs::read_to_string(&report_path)?;
                    println!("Verification Report:\n{rep}\n");
                }
                if fallbacks_path.exists() {
                    let fbs = fs::read_to_string(&fallbacks_path)?;
                    println!("Migration Debt (Fallbacks):\n{fbs}\n");
                }
                if arch_path.exists() {
                    let arch = fs::read_to_string(&arch_path)?;
                    println!("Architecture Summary:\n{arch}\n");
                }
            }
        }
        Commands::Eval { fixtures, output } => {
            println!(
                "🏆 Running Project Exodus Benchmark Suite on `{}`...",
                fixtures.display()
            );
            let evaluator = Evaluator::new();
            let summary = evaluator.run_full_benchmark_suite(&fixtures).await?;

            fs::create_dir_all(&output)?;
            let md_report = evaluator.to_markdown(&summary);
            let csv_report = evaluator.to_csv(&summary);
            let json_report = serde_json::to_string_pretty(&summary)?;

            fs::write(output.join("evaluation_scorecard.md"), &md_report)?;
            fs::write(output.join("evaluation_scorecard.csv"), &csv_report)?;
            fs::write(output.join("evaluation_scorecard.json"), &json_report)?;

            if cli.json {
                println!("{}", json_report);
            } else {
                println!("\n{md_report}");
                println!("📁 Scorecards saved to {}", output.display());
            }
        }
        Commands::Units(unit_cmd) => match unit_cmd {
            UnitsCommands::List { source } => {
                let parser = PythonParser::new();
                let parsed = parser.parse_repository(&source)?;
                let graph = SemanticGraph::from_parsed_repository(&parsed);
                if cli.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&graph.verification_units())?
                    );
                } else {
                    let boundaries = graph.verification_units();
                    println!(
                        "📦 Extracted Verification Boundaries ({} total):",
                        boundaries.len()
                    );
                    for b in &boundaries {
                        let unit_id = unit_id_for(b);
                        if b.is_cluster() {
                            println!("   • [Cluster, {} nodes] {unit_id}", b.node_ids().len());
                        } else {
                            println!("   • [Unit] {unit_id}");
                        }
                    }
                }
            }
            UnitsCommands::Show { unit_id, source } => {
                let parser = PythonParser::new();
                let parsed = parser.parse_repository(&source)?;
                let graph = SemanticGraph::from_parsed_repository(&parsed);
                let boundary = graph
                    .verification_units()
                    .into_iter()
                    .find(|b| unit_id_for(b) == unit_id);
                match boundary {
                    None => {
                        eprintln!("❌ Unknown unit `{unit_id}`. Run `exodus units list {}` to see valid IDs.", source.display());
                        std::process::exit(1);
                    }
                    Some(b) => {
                        let contract = load_fixture_contract(&source, &unit_id);
                        if cli.json {
                            println!(
                                "{}",
                                serde_json::to_string_pretty(&serde_json::json!({
                                    "unit_id": unit_id,
                                    "boundary": b,
                                    "contract_grounded": contract.is_some(),
                                }))?
                            );
                        } else {
                            println!("🔍 Unit `{unit_id}`");
                            println!(
                                "   - Boundary kind: {}",
                                if b.is_cluster() { "cluster" } else { "unit" }
                            );
                            if b.is_cluster() {
                                println!("   - Members: {}", b.node_ids().join(", "));
                            }
                            for node_id in b.node_ids() {
                                if let Some(n) = graph.get_node(&node_id) {
                                    println!(
                                        "   - [{:?}] {} — risk {}",
                                        n.kind, n.qualified_name, n.risk_score
                                    );
                                }
                            }
                            println!(
                                "   - Grounded contract: {}",
                                if contract.is_some() {
                                    "yes"
                                } else {
                                    "no (run `units verify` — it will still compile-check but cannot claim behavioral verification)"
                                }
                            );
                        }
                    }
                }
            }
            UnitsCommands::Verify { unit_id, source } => {
                let parser = PythonParser::new();
                let parsed = parser.parse_repository(&source)?;
                let graph = SemanticGraph::from_parsed_repository(&parsed);
                let boundary = graph
                    .verification_units()
                    .into_iter()
                    .find(|b| unit_id_for(b) == unit_id);
                match boundary {
                    None => {
                        eprintln!("❌ Unknown unit `{unit_id}`. Run `exodus units list {}` to see valid IDs.", source.display());
                        std::process::exit(1);
                    }
                    Some(b) => {
                        let contract = load_fixture_contract(&source, &unit_id);
                        let case_engine = CaseEngine::new(Path::new(".exodus").join("knowledge"));
                        let ctx = UnitGateContext {
                            run_id: "cli-ad-hoc",
                            repo: &parsed,
                            graph: &graph,
                            contracts_dir: Path::new(".exodus").join("contracts"),
                            work_dir: Path::new(".exodus")
                                .join("work")
                                .join(unit_id.replace(['+', ':'], "_")),
                            case_engine: &case_engine,
                            contract,
                            repair_bounds: AgentBounds::default(),
                        };
                        let (result, _) = run_unit_gate(&b, &ctx).await?;
                        if cli.json {
                            println!("{}", serde_json::to_string_pretty(&result)?);
                        } else {
                            let icon = match result.outcome {
                                exodus_core::MigrationOutcome::Verified => "✅",
                                exodus_core::MigrationOutcome::Compatible => "🟡",
                                exodus_core::MigrationOutcome::Degraded => "⚠️",
                                exodus_core::MigrationOutcome::Blocked => "❌",
                            };
                            println!("{icon} Unit `{unit_id}`: {}", result.outcome);
                            println!("   - Compiled: {}", result.compiled);
                            println!(
                                "   - Assertions: {}/{} passed",
                                result.assertions_passed, result.assertions_total
                            );
                            if let Some(case_id) = &result.case_id {
                                println!("   - Localized to Migration Case: {case_id}");
                            }
                            println!(
                                "📁 Artifacts: .exodus/contracts/{unit_id}/{{behavioral-contract,verification}}.json"
                            );
                        }
                    }
                }
            }
        },
        Commands::Contracts(contract_cmd) => match contract_cmd {
            ContractsCommands::Show { unit_id, source } => {
                let contract_path = Path::new(".exodus")
                    .join("contracts")
                    .join(&unit_id)
                    .join("behavioral-contract.json");
                if contract_path.exists() {
                    let content = fs::read_to_string(contract_path)?;
                    println!("{content}");
                } else if let Some(grounded) = load_fixture_contract(&source, &unit_id) {
                    // Not yet verified by a real gate run, but a grounded (never fabricated)
                    // source contract exists for this unit — show it labeled as such rather than
                    // fabricating a "generated" one.
                    if cli.json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "unit_id": unit_id,
                                "status": "grounded_but_not_yet_verified",
                                "contract": grounded,
                            }))?
                        );
                    } else {
                        println!("ℹ️  A grounded contract exists for `{unit_id}` but it has not been run through `units verify` yet:");
                        println!("{}", serde_json::to_string_pretty(&grounded)?);
                    }
                } else if cli.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "unit_id": unit_id,
                            "verification_status": "not_generated",
                            "reason": "no behavioral-contract.json has been generated for this unit yet",
                        }))?
                    );
                } else {
                    println!("⚠️  No behavioral contract has been generated for `{unit_id}` yet.");
                    println!("   Run `exodus units verify {unit_id}` to generate and verify one.");
                }
            }
            ContractsCommands::Verify { unit_id, source } => {
                let parser = PythonParser::new();
                let parsed = parser.parse_repository(&source)?;
                let graph = SemanticGraph::from_parsed_repository(&parsed);
                let boundary = graph
                    .verification_units()
                    .into_iter()
                    .find(|b| unit_id_for(b) == unit_id);
                match boundary {
                    None => {
                        eprintln!("❌ Unknown unit `{unit_id}`.");
                        std::process::exit(1);
                    }
                    Some(b) => {
                        let contract = load_fixture_contract(&source, &unit_id);
                        if contract.is_none() {
                            eprintln!("⚠️  No grounded contract for `{unit_id}` — nothing to verify assertions against (compile-only check via `units verify`).");
                            std::process::exit(1);
                        }
                        let case_engine = CaseEngine::new(Path::new(".exodus").join("knowledge"));
                        let ctx = UnitGateContext {
                            run_id: "cli-ad-hoc",
                            repo: &parsed,
                            graph: &graph,
                            contracts_dir: Path::new(".exodus").join("contracts"),
                            work_dir: Path::new(".exodus")
                                .join("work")
                                .join(unit_id.replace(['+', ':'], "_")),
                            case_engine: &case_engine,
                            contract,
                            repair_bounds: AgentBounds::default(),
                        };
                        let (result, _) = run_unit_gate(&b, &ctx).await?;
                        if cli.json {
                            println!("{}", serde_json::to_string_pretty(&result)?);
                        } else if result.assertions_failed == 0 && result.assertions_passed > 0 {
                            println!(
                                "✅ Behavioral contract for `{unit_id}`: {}/{} ASSERTIONS PASSED.",
                                result.assertions_passed, result.assertions_total
                            );
                        } else {
                            println!(
                                "❌ Behavioral contract for `{unit_id}`: {}/{} assertions passed (case: {})",
                                result.assertions_passed,
                                result.assertions_total,
                                result.case_id.as_deref().unwrap_or("none")
                            );
                        }
                    }
                }
            }
        },
        Commands::Cases(case_cmd) => {
            let engine = CaseEngine::new(Path::new(".exodus").join("knowledge"));
            match case_cmd {
                CasesCommands::List => {
                    let cases = engine.list_cases()?;
                    println!("📚 Governed Migration Cases ({} total):", cases.len());
                    for c in cases {
                        println!(
                            "   • [{:?}] {} - {} ({})",
                            c.status,
                            c.case_id,
                            c.failure_description,
                            c.confidence_statement()
                        );
                    }
                }
                CasesCommands::Show { case_id } => {
                    let cases = engine.list_cases()?;
                    if let Some(c) = cases.iter().find(|c| c.case_id == case_id) {
                        println!("{}", serde_json::to_string_pretty(c)?);
                    } else {
                        eprintln!("❌ Case not found: {case_id}");
                    }
                }
                CasesCommands::Search { query } => {
                    // Free-text structural search across every promoted case, not filtered to a
                    // single hardcoded failure category — a case is a match if the query text
                    // appears in its unit ID, failure description, or fingerprint.
                    let all_cases = engine.list_cases()?;
                    let query_lower = query.to_lowercase();
                    let matches: Vec<_> = all_cases
                        .iter()
                        .filter(|c| c.status == exodus_case::CaseStatus::Promoted)
                        .filter(|c| {
                            c.failure_description.to_lowercase().contains(&query_lower)
                                || c.unit_id
                                    .as_deref()
                                    .unwrap_or("")
                                    .to_lowercase()
                                    .contains(&query_lower)
                                || c.structural_fingerprint
                                    .to_lowercase()
                                    .contains(&query_lower)
                        })
                        .collect();
                    if cli.json {
                        println!("{}", serde_json::to_string_pretty(&matches)?);
                    } else {
                        println!(
                            "🔍 Found {} promoted case(s) matching query `{query}`:",
                            matches.len()
                        );
                        for m in matches {
                            println!("   • [{}] {}", m.case_id, m.confidence_statement());
                        }
                    }
                }
                CasesCommands::Review { case_id } => {
                    let cases = engine.list_cases()?;
                    match cases.iter().find(|c| c.case_id == case_id) {
                        None => {
                            eprintln!("❌ Case not found: {case_id}");
                            std::process::exit(1);
                        }
                        Some(c) => {
                            println!("🧐 Case `{case_id}` — {:?}", c.status);
                            println!(
                                "   - Unit: {}",
                                c.unit_id.as_deref().unwrap_or("(none recorded)")
                            );
                            println!("   - Failure category: {:?}", c.failure_category);
                            println!("   - Description: {}", c.failure_description);
                            if let Some(d) = &c.compiler_diagnostic {
                                println!("   - Diagnostic evidence:\n{d}");
                            }
                            if let Some(a) = &c.failed_assertion {
                                println!("   - Failed assertion: {a}");
                            }
                            println!("   - {}", c.confidence_statement());
                            println!(
                                "   Run `exodus cases approve {case_id}` or `exodus cases reject {case_id}` to record a review decision."
                            );
                        }
                    }
                }
                CasesCommands::Approve { case_id, approver } => {
                    let approved = engine.approve_case(&case_id, &approver)?;
                    println!(
                        "✅ Case `{}` approved and PROMOTED by `{}`",
                        approved.case_id, approver
                    );
                }
                CasesCommands::Reject { case_id } => {
                    let rejected = engine.reject_case(&case_id)?;
                    println!("🚫 Case `{}` DEPRECATED.", rejected.case_id);
                }
                CasesCommands::Test { mode } => {
                    let all_cases = engine.list_cases()?;
                    let promoted: Vec<_> = all_cases
                        .iter()
                        .filter(|c| c.status == exodus_case::CaseStatus::Promoted)
                        .collect();

                    match mode {
                        TestMode::Replay => {
                            // Schema validation (already guaranteed by successful deserialization
                            // above) + fingerprint recomputation from each case's own stored ESG
                            // subgraph + a retrieval/ranking sanity check — no network inference,
                            // no target generation, matching "replay" per the master prompt.
                            let mut consistent = 0usize;
                            let mut inconsistent = Vec::new();
                            let mut retrievable = 0usize;
                            for c in &promoted {
                                let fingerprint_ok = match &c.esg_subgraph {
                                    Some(sg_json) => {
                                        match serde_json::from_str::<SemanticGraph>(sg_json) {
                                            Ok(sg) => {
                                                let recomputed = CaseEngine::compute_fingerprint(
                                                    &c.failure_category,
                                                    Some(&sg),
                                                    &c.source_language,
                                                    &c.target_language,
                                                );
                                                recomputed == c.structural_fingerprint
                                            }
                                            Err(_) => false,
                                        }
                                    }
                                    None => true, // nothing to recompute against; not a mismatch
                                };
                                if fingerprint_ok {
                                    consistent += 1;
                                } else {
                                    inconsistent.push(c.case_id.clone());
                                }
                                let found = engine.search_promoted_cases(
                                    &c.structural_fingerprint,
                                    &c.failure_category,
                                )?;
                                if found.iter().any(|f| f.case_id == c.case_id) {
                                    retrievable += 1;
                                }
                            }
                            let report = serde_json::json!({
                                "mode": "replay",
                                "promoted_cases": promoted.len(),
                                "fingerprint_consistent": consistent,
                                "fingerprint_inconsistent": inconsistent,
                                "retrievable_via_search": retrievable,
                            });
                            if cli.json {
                                println!("{}", serde_json::to_string_pretty(&report)?);
                            } else {
                                println!(
                                    "🧪 Replay: {consistent}/{} promoted case(s) have a consistent structural fingerprint; {retrievable}/{} are retrievable via structural search.",
                                    promoted.len(), promoted.len()
                                );
                                if !inconsistent.is_empty() {
                                    println!(
                                        "   ⚠️  Inconsistent fingerprints: {}",
                                        inconsistent.join(", ")
                                    );
                                }
                            }
                        }
                        TestMode::Verify => {
                            // Full verify mode regenerates the target and re-runs the real gate —
                            // this pass does not yet generate a regression fixture on promotion
                            // (out of scope for this milestone), so a promoted case without one is
                            // reported as not-yet-re-verifiable rather than silently counted as a
                            // pass.
                            let with_fixture = promoted
                                .iter()
                                .filter(|c| c.regression_fixture_path.is_some())
                                .count();
                            let report = serde_json::json!({
                                "mode": "verify",
                                "promoted_cases": promoted.len(),
                                "with_regression_fixture": with_fixture,
                                "re_verified": 0,
                                "note": "regression-fixture generation on case promotion is not yet implemented; verify mode has nothing to re-run against for these cases",
                            });
                            if cli.json {
                                println!("{}", serde_json::to_string_pretty(&report)?);
                            } else {
                                println!(
                                    "🧪 Verify: {with_fixture}/{} promoted case(s) have a regression fixture to re-run.",
                                    promoted.len()
                                );
                                if with_fixture == 0 && !promoted.is_empty() {
                                    println!("   ⚠️  Regression-fixture generation on promotion is not yet implemented — nothing to re-verify.");
                                }
                            }
                        }
                    }
                }
            }
        }
        Commands::Worktree(wt_cmd) => {
            let manager = WorktreeManager::new(".");
            match wt_cmd {
                WorktreeCommands::List => {
                    let leases = manager.list_leases()?;
                    println!("🌳 Active Migration Worktrees ({} total):", leases.len());
                    for l in leases {
                        println!(
                            "   • Task `{}` (Run: {}) -> {} [{:?}]",
                            l.task_id,
                            l.run_id,
                            l.worktree_path.display(),
                            l.status
                        );
                    }
                }
                WorktreeCommands::Inspect { task_id } => {
                    let leases = manager.list_leases()?;
                    if let Some(l) = leases.iter().find(|l| l.task_id == task_id) {
                        println!("{}", serde_json::to_string_pretty(l)?);
                    } else {
                        eprintln!("❌ Worktree task not found: {task_id}");
                    }
                }
                WorktreeCommands::Resume { task_id } => {
                    let leases = manager.list_leases()?;
                    match leases.iter().find(|l| l.task_id == task_id) {
                        None => {
                            eprintln!("❌ No lease found for task `{task_id}`.");
                            std::process::exit(1);
                        }
                        Some(l) => {
                            println!("🔄 Task `{task_id}` — status {:?}", l.status);
                            println!("   - Worktree: {}", l.worktree_path.display());
                            println!("   - Branch: {}", l.branch_name);
                            println!(
                                "   - Exists on disk: {}",
                                if l.worktree_path.exists() {
                                    "yes"
                                } else {
                                    "no — recovery required"
                                }
                            );
                            if let Some(commit) = &l.current_commit {
                                println!("   - Last verified commit: {commit}");
                            }
                        }
                    }
                }
                WorktreeCommands::Preserve { task_id } => {
                    let leases = manager.list_leases()?;
                    match leases.into_iter().find(|l| l.task_id == task_id) {
                        None => {
                            eprintln!("❌ No lease found for task `{task_id}`.");
                            std::process::exit(1);
                        }
                        Some(mut l) => {
                            l.status = exodus_worktree::WorktreeStatus::DirtyReviewRequired;
                            let runs_dir = Path::new(".exodus").join("runs").join(&l.run_id);
                            fs::create_dir_all(&runs_dir)?;
                            fs::write(
                                runs_dir.join("worktree.json"),
                                serde_json::to_string_pretty(&l)?,
                            )?;
                            println!("🛡️ Task `{task_id}` marked DirtyReviewRequired and preserved for human review at {}", l.worktree_path.display());
                        }
                    }
                }
                WorktreeCommands::Cleanup { task_id } => {
                    let removed = manager.cleanup_lease(&task_id, false)?;
                    if removed {
                        println!("🧹 Cleaned up worktree lease for task `{task_id}`");
                    } else {
                        println!("⚠️  Worktree for task `{task_id}` was dirty or not found — preserved, not removed. Use `worktree discard --confirm {task_id}` to force.");
                    }
                }
                WorktreeCommands::Discard { task_id, confirm } => {
                    if task_id == confirm {
                        let removed = manager.cleanup_lease(&task_id, true)?;
                        if removed {
                            println!("🗑️ Force discarded worktree lease `{task_id}`");
                        } else {
                            eprintln!("❌ No lease found for task `{task_id}`.");
                            std::process::exit(1);
                        }
                    } else {
                        eprintln!(
                            "❌ Confirmation mismatch: expected `{task_id}`, got `{confirm}`"
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
