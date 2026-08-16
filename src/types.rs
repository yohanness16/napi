use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported framework architectures detected in the repository
#[napi(string_enum)]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameworkType {
    NextAppRouter,
    NextPagesRouter,
    Remix,
    Vite,
    React,
    Vue,
    NestJs,
    Express,
    Generic,
}

/// Target architectural styles for repository transformation
#[napi(string_enum)]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchitectureTarget {
    FeatureBased,
    DomainDrivenDesign,
    Layered,
    Custom,
}

/// Supported naming conventions for files and identifiers
#[napi(string_enum)]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamingConvention {
    KebabCase,
    PascalCase,
    CamelCase,
    SnakeCase,
    Preserve,
}

/// Kinds of import / export statements detected via AST
#[napi(string_enum)]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportKind {
    StaticImport,
    DynamicImport,
    RequireCall,
    ExportFrom,
    ExportAll,
    TypeOnlyImport,
}

/// Exact byte span of an AST token or specifier
#[napi(object)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpanInfo {
    pub start: u32,
    pub end: u32,
    pub line: u32,
    pub column: u32,
}

/// Information about an imported specifier in a source file
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDeclarationInfo {
    pub specifier: String,
    pub raw_specifier: String,
    pub span: SpanInfo,
    pub kind: ImportKind,
    pub resolved_path: Option<String>,
    pub is_external: bool,
    pub is_type_only: bool,
}

/// Framework boundary information for a specific file
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkBoundaryInfo {
    pub is_boundary: bool,
    pub is_protected_route: bool,
    pub boundary_type: String,
    pub description: String,
    pub directive: Option<String>,
}

/// Metadata and analysis for a single file in the repository
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub relative_path: String,
    pub file_name: String,
    pub extension: String,
    pub size_bytes: u32,
    pub line_count: u32,
    pub framework_boundary: FrameworkBoundaryInfo,
    pub imports: Vec<ImportDeclarationInfo>,
    pub exported_symbols: Vec<String>,
}

/// Detected tsconfig path mapping configuration
#[napi(object)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TsConfigInfo {
    pub base_url: Option<String>,
    pub paths: HashMap<String, Vec<String>>,
}

/// Configuration options for scanning a repository
#[napi(object)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanConfig {
    pub root_path: String,
    pub ignore_patterns: Option<Vec<String>>,
    pub tsconfig_path: Option<String>,
}

/// Circular dependency cycle path
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularCycle {
    pub files: Vec<String>,
    pub cycle_length: u32,
}

/// Node in the repository dependency graph
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraphNode {
    pub file_path: String,
    pub relative_path: String,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
    pub fan_in: u32,
    pub fan_out: u32,
    pub is_circular: bool,
    pub is_orphan: bool,
}

/// Complete dependency graph analysis result
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraphResult {
    pub total_nodes: u32,
    pub total_edges: u32,
    pub nodes: Vec<DependencyGraphNode>,
    pub circular_cycles: Vec<CircularCycle>,
    pub orphan_files: Vec<String>,
}

/// Full scan analysis result returned to Node.js
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryScanResult {
    pub root_path: String,
    pub framework: FrameworkType,
    pub framework_description: String,
    pub total_files: u32,
    pub total_lines: u32,
    pub files: Vec<FileInfo>,
    pub dependency_graph: DependencyGraphResult,
    pub tsconfig: TsConfigInfo,
}

/// Planned file movement or renaming operation
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMoveAction {
    pub original_path: String,
    pub original_relative_path: String,
    pub new_path: String,
    pub new_relative_path: String,
    pub reason: String,
    pub is_protected_framework_file: bool,
}

/// Planned AST byte patch operation for an import/export specifier
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchAction {
    pub file_path: String,
    pub span_start: u32,
    pub span_end: u32,
    pub original_specifier: String,
    pub replacement_specifier: String,
    pub reason: String,
}

/// Configuration options for generating an architectural refactor plan
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanConfig {
    pub root_path: String,
    pub target_architecture: ArchitectureTarget,
    pub naming_convention: NamingConvention,
    pub custom_feature_mappings: Option<HashMap<String, String>>,
    pub tsconfig_path: Option<String>,
}

/// Summary metrics of a refactor plan
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorSummary {
    pub total_files_moved: u32,
    pub total_imports_patched: u32,
    pub total_protected_files: u32,
    pub target_architecture: ArchitectureTarget,
    pub naming_convention: NamingConvention,
}

/// Complete architectural refactoring plan
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactorPlan {
    pub root_path: String,
    pub target_architecture: ArchitectureTarget,
    pub naming_convention: NamingConvention,
    pub file_moves: Vec<FileMoveAction>,
    pub patches: Vec<PatchAction>,
    pub protected_files: Vec<String>,
    pub summary: RefactorSummary,
}

/// Diff hunk for a modified file
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub header: String,
    pub lines: Vec<String>,
}

/// Unified diff for a single file affected by the refactoring
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub file_path: String,
    pub relative_path: String,
    pub is_new_file: bool,
    pub is_deleted_file: bool,
    pub is_moved: bool,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub unified_diff: String,
    pub additions: u32,
    pub deletions: u32,
    pub hunks: Vec<DiffHunk>,
}

/// Diff preview result for user inspection
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffPreviewResult {
    pub total_files_changed: u32,
    pub total_additions: u32,
    pub total_deletions: u32,
    pub file_diffs: Vec<FileDiff>,
}

/// Options for applying a refactor transaction
#[napi(object)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApplyOptions {
    pub dry_run: Option<bool>,
    pub force: Option<bool>,
    pub skip_git_check: Option<bool>,
    pub journal_dir: Option<String>,
}

/// Result of applying a refactor transaction
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub success: bool,
    pub files_moved: u32,
    pub files_patched: u32,
    pub journal_path: Option<String>,
    pub transaction_id: String,
    pub message: String,
}

/// Result of rolling back a refactor transaction
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub success: bool,
    pub restored_files_count: u32,
    pub transaction_id: String,
    pub message: String,
}

/// Git status information for guardrails
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatusResult {
    pub is_git_repo: bool,
    pub is_clean: bool,
    pub git_root: Option<String>,
    pub branch: Option<String>,
    pub modified_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub warning: Option<String>,
}

/// Single code clone location
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeCloneInstance {
    pub file_path: String,
    pub relative_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub start_byte: u32,
    pub end_byte: u32,
    pub function_name: Option<String>,
    pub code_snippet: String,
}

/// Cluster of duplicated code blocks across the codebase
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneCluster {
    pub cluster_id: String,
    pub hash: String,
    pub instance_count: u32,
    pub lines_per_instance: u32,
    pub potential_lines_saved: u32,
    pub ast_node_count: u32,
    pub instances: Vec<CodeCloneInstance>,
    pub suggested_module_name: String,
    pub suggested_target_path: String,
}

/// Configuration for code clone and deduplication detection
#[napi(object)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CloneDetectionConfig {
    pub root_path: String,
    pub min_lines: Option<u32>,
    pub min_ast_nodes: Option<u32>,
    pub ignore_patterns: Option<Vec<String>>,
}

/// Result of code clone detection
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneDetectionResult {
    pub total_clones_found: u32,
    pub total_clusters: u32,
    pub total_lines_saved: u32,
    pub clusters: Vec<CloneCluster>,
}

/// Saved transaction journal entry for safety and rollbacks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub transaction_id: String,
    pub timestamp: String,
    pub root_path: String,
    pub target_architecture: ArchitectureTarget,
    pub naming_convention: NamingConvention,
    pub file_moves: Vec<FileMoveAction>,
    pub patches: Vec<PatchAction>,
    pub original_files_backup: HashMap<String, String>,
    pub created_files: Vec<String>,
    pub created_directories: Vec<String>,
    pub status: JournalStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalStatus {
    Applied,
    RolledBack,
    Failed,
}
