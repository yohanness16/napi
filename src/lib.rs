pub mod ast_graph;
pub mod clones;
pub mod patcher;
pub mod planner;
pub mod scanner;
pub mod transaction;
pub mod types;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::Path;

use ast_graph::AstGraphAnalyzer;
use clones::CloneDetector;
use planner::ArchitecturalPlanner;
use scanner::Scanner;
use transaction::TransactionEngine;
use types::*;

/// Scan a repository to detect frameworks, boundary files, imports, and AST dependency graph
#[napi]
pub fn scan_repository(config: ScanConfig) -> Result<RepositoryScanResult> {
    let mut scan_result = Scanner::scan(&config).map_err(|e| Error::from_reason(e))?;
    let root = Path::new(&scan_result.root_path);

    let dep_graph = AstGraphAnalyzer::build_dependency_graph(
        &mut scan_result.files,
        root,
        &scan_result.tsconfig,
    );

    scan_result.dependency_graph = dep_graph;
    Ok(scan_result)
}

/// Generate an architectural refactoring plan with naming normalization and import patching
#[napi]
pub fn generate_plan(config: PlanConfig) -> Result<RefactorPlan> {
    let scan_config = ScanConfig {
        root_path: config.root_path.clone(),
        ignore_patterns: None,
        tsconfig_path: config.tsconfig_path.clone(),
    };

    let mut scan_result = Scanner::scan(&scan_config).map_err(|e| Error::from_reason(e))?;
    let root = Path::new(&scan_result.root_path);

    let _ = AstGraphAnalyzer::build_dependency_graph(
        &mut scan_result.files,
        root,
        &scan_result.tsconfig,
    );

    let plan = ArchitecturalPlanner::plan(
        &config,
        &scan_result.files,
        scan_result.framework,
        &scan_result.tsconfig,
    )
    .map_err(|e| Error::from_reason(e))?;

    Ok(plan)
}

/// Generate a unified diff preview for a refactor plan without modifying disk files
#[napi]
pub fn preview_diff(plan: RefactorPlan) -> Result<DiffPreviewResult> {
    TransactionEngine::preview_diff(&plan).map_err(|e| Error::from_reason(e))
}

/// Apply a refactoring plan atomically with git guardrails and rollback journal creation
#[napi]
pub fn apply_refactor(plan: RefactorPlan, options: Option<ApplyOptions>) -> Result<ApplyResult> {
    let opts = options.unwrap_or_default();
    TransactionEngine::apply(&plan, &opts).map_err(|e| Error::from_reason(e))
}

/// Roll back a refactor transaction using the `.refactor-journal.json` file
#[napi]
pub fn rollback_refactor(journal_path: Option<String>, root_path: Option<String>) -> Result<RollbackResult> {
    TransactionEngine::rollback(journal_path.as_deref(), root_path.as_deref())
        .map_err(|e| Error::from_reason(e))
}

/// Detect duplicated code patterns and AST clones across the repository
#[napi]
pub fn detect_clones(config: CloneDetectionConfig) -> Result<CloneDetectionResult> {
    CloneDetector::detect_clones(&config).map_err(|e| Error::from_reason(e))
}

/// Check the Git status of the repository for clean working tree validation
#[napi]
pub fn get_git_status(root_path: String) -> Result<GitStatusResult> {
    Ok(TransactionEngine::get_git_status(&root_path))
}
