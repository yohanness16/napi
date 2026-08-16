use crate::scanner::SUPPORTED_EXTENSIONS;
use crate::types::{
    CircularCycle, DependencyGraphNode, DependencyGraphResult, FileInfo, ImportDeclarationInfo,
    ImportKind, SpanInfo, TsConfigInfo,
};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, Expression, Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

pub struct AstGraphAnalyzer;

impl AstGraphAnalyzer {
    /// Parse a source file and extract all import declarations, export specifiers, and exported symbols
    pub fn parse_file_imports(
        file_path: &Path,
        root_path: &Path,
        tsconfig: &TsConfigInfo,
    ) -> (Vec<ImportDeclarationInfo>, Vec<String>) {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return (Vec::new(), Vec::new()),
        };

        let allocator = Allocator::default();
        let source_type = SourceType::from_path(file_path).unwrap_or_default();
        let ret = Parser::new(&allocator, &content, source_type).parse();

        let mut imports = Vec::new();
        let mut exports = Vec::new();

        let line_starts = Self::compute_line_starts(&content);

        for stmt in &ret.program.body {
            match stmt {
                Statement::ImportDeclaration(import_decl) => {
                    let specifier = import_decl.source.value.to_string();
                    let raw_specifier = import_decl.source.raw.as_deref().unwrap_or(&specifier).to_string();
                    let span = import_decl.source.span;

                    let (line, column) = Self::offset_to_line_col(span.start, &line_starts);
                    let is_type_only = import_decl.import_kind.is_type();

                    let kind = if is_type_only {
                        ImportKind::TypeOnlyImport
                    } else {
                        ImportKind::StaticImport
                    };

                    let resolved = Self::resolve_module(
                        &specifier,
                        file_path,
                        root_path,
                        tsconfig,
                    );

                    let is_external = resolved.is_none() && !specifier.starts_with('.') && !specifier.starts_with('/');

                    imports.push(ImportDeclarationInfo {
                        specifier,
                        raw_specifier,
                        span: SpanInfo {
                            start: span.start,
                            end: span.end,
                            line,
                            column,
                        },
                        kind,
                        resolved_path: resolved,
                        is_external,
                        is_type_only,
                    });
                }
                Statement::ExportNamedDeclaration(export_decl) => {
                    if let Some(ref source) = export_decl.source {
                        let specifier = source.value.to_string();
                        let raw_specifier = source.raw.as_deref().unwrap_or(&specifier).to_string();
                        let span = source.span;
                        let (line, column) = Self::offset_to_line_col(span.start, &line_starts);

                        let resolved = Self::resolve_module(
                            &specifier,
                            file_path,
                            root_path,
                            tsconfig,
                        );

                        let is_external = resolved.is_none() && !specifier.starts_with('.') && !specifier.starts_with('/');

                        imports.push(ImportDeclarationInfo {
                            specifier,
                            raw_specifier,
                            span: SpanInfo {
                                start: span.start,
                                end: span.end,
                                line,
                                column,
                            },
                            kind: ImportKind::ExportFrom,
                            resolved_path: resolved,
                            is_external,
                            is_type_only: export_decl.export_kind.is_type(),
                        });
                    }

                    for specifier in &export_decl.specifiers {
                        exports.push(specifier.exported.name().to_string());
                    }
                }
                Statement::ExportAllDeclaration(export_all) => {
                    let specifier = export_all.source.value.to_string();
                    let raw_specifier = export_all.source.raw.as_deref().unwrap_or(&specifier).to_string();
                    let span = export_all.source.span;
                    let (line, column) = Self::offset_to_line_col(span.start, &line_starts);

                    let resolved = Self::resolve_module(
                        &specifier,
                        file_path,
                        root_path,
                        tsconfig,
                    );

                    let is_external = resolved.is_none() && !specifier.starts_with('.') && !specifier.starts_with('/');

                    imports.push(ImportDeclarationInfo {
                        specifier,
                        raw_specifier,
                        span: SpanInfo {
                            start: span.start,
                            end: span.end,
                            line,
                            column,
                        },
                        kind: ImportKind::ExportAll,
                        resolved_path: resolved,
                        is_external,
                        is_type_only: export_all.export_kind.is_type(),
                    });
                }
                Statement::ExportDefaultDeclaration(_) => {
                    exports.push("default".to_string());
                }
                _ => {
                    Self::extract_dynamic_imports_from_stmt(stmt, file_path, root_path, tsconfig, &line_starts, &mut imports);
                }
            }
        }

        (imports, exports)
    }

    /// Recursively search AST statements for dynamic import() or require() expressions
    fn extract_dynamic_imports_from_stmt(
        stmt: &Statement,
        file_path: &Path,
        root_path: &Path,
        tsconfig: &TsConfigInfo,
        line_starts: &[usize],
        imports: &mut Vec<ImportDeclarationInfo>,
    ) {
        match stmt {
            Statement::ExpressionStatement(expr_stmt) => {
                Self::extract_dynamic_imports_from_expr(&expr_stmt.expression, file_path, root_path, tsconfig, line_starts, imports);
            }
            Statement::VariableDeclaration(var_decl) => {
                for decl in &var_decl.declarations {
                    if let Some(ref init) = decl.init {
                        Self::extract_dynamic_imports_from_expr(init, file_path, root_path, tsconfig, line_starts, imports);
                    }
                }
            }
            _ => {}
        }
    }

    fn extract_dynamic_imports_from_expr(
        expr: &Expression,
        file_path: &Path,
        root_path: &Path,
        tsconfig: &TsConfigInfo,
        line_starts: &[usize],
        imports: &mut Vec<ImportDeclarationInfo>,
    ) {
        match expr {
            Expression::ImportExpression(import_expr) => {
                if let Expression::StringLiteral(ref s) = import_expr.source {
                    let specifier = s.value.to_string();
                    let raw_specifier = s.raw.as_deref().unwrap_or(&specifier).to_string();
                    let span = s.span;
                    let (line, column) = Self::offset_to_line_col(span.start, line_starts);

                    let resolved = Self::resolve_module(&specifier, file_path, root_path, tsconfig);
                    let is_external = resolved.is_none() && !specifier.starts_with('.') && !specifier.starts_with('/');

                    imports.push(ImportDeclarationInfo {
                        specifier,
                        raw_specifier,
                        span: SpanInfo {
                            start: span.start,
                            end: span.end,
                            line,
                            column,
                        },
                        kind: ImportKind::DynamicImport,
                        resolved_path: resolved,
                        is_external,
                        is_type_only: false,
                    });
                }
            }
            Expression::CallExpression(call_expr) => {
                if let Expression::Identifier(ref ident) = call_expr.callee {
                    if ident.name == "require" && !call_expr.arguments.is_empty() {
                        if let Some(Argument::StringLiteral(ref s)) = call_expr.arguments.first() {
                            let specifier = s.value.to_string();
                            let raw_specifier = s.raw.as_deref().unwrap_or(&specifier).to_string();
                            let span = s.span;
                            let (line, column) = Self::offset_to_line_col(span.start, line_starts);

                            let resolved = Self::resolve_module(&specifier, file_path, root_path, tsconfig);
                            let is_external = resolved.is_none() && !specifier.starts_with('.') && !specifier.starts_with('/');

                            imports.push(ImportDeclarationInfo {
                                specifier,
                                raw_specifier,
                                span: SpanInfo {
                                    start: span.start,
                                    end: span.end,
                                    line,
                                    column,
                                },
                                kind: ImportKind::RequireCall,
                                resolved_path: resolved,
                                is_external,
                                is_type_only: false,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Resolve an import specifier to an absolute filesystem path
    pub fn resolve_module(
        specifier: &str,
        importer_path: &Path,
        root_path: &Path,
        tsconfig: &TsConfigInfo,
    ) -> Option<String> {
        if specifier.is_empty() {
            return None;
        }

        if specifier.starts_with("./") || specifier.starts_with("../") {
            let importer_dir = importer_path.parent().unwrap_or(root_path);
            let candidate_path = importer_dir.join(specifier);
            return Self::try_resolve_file_candidate(&candidate_path);
        }

        for (alias_pattern, target_patterns) in &tsconfig.paths {
            let prefix = alias_pattern.trim_end_matches('*');
            if specifier.starts_with(prefix) {
                let suffix = &specifier[prefix.len()..];
                for target_pattern in target_patterns {
                    let target_prefix = target_pattern.trim_end_matches('*');
                    let base_dir = if let Some(ref base) = tsconfig.base_url {
                        root_path.join(base)
                    } else {
                        root_path.to_path_buf()
                    };

                    let candidate = base_dir.join(format!("{}{}", target_prefix, suffix));
                    if let Some(resolved) = Self::try_resolve_file_candidate(&candidate) {
                        return Some(resolved);
                    }
                }
            }
        }

        if let Some(ref base) = tsconfig.base_url {
            let candidate = root_path.join(base).join(specifier);
            if let Some(resolved) = Self::try_resolve_file_candidate(&candidate) {
                return Some(resolved);
            }
        }

        None
    }

    fn try_resolve_file_candidate(candidate: &Path) -> Option<String> {
        if candidate.is_file() {
            return candidate.canonicalize().ok().map(|p| p.to_string_lossy().to_string());
        }

        for ext in SUPPORTED_EXTENSIONS {
            let with_ext = candidate.with_extension(ext);
            if with_ext.is_file() {
                return with_ext.canonicalize().ok().map(|p| p.to_string_lossy().to_string());
            }
        }

        if candidate.is_dir() {
            for ext in SUPPORTED_EXTENSIONS {
                let index_file = candidate.join(format!("index.{}", ext));
                if index_file.is_file() {
                    return index_file.canonicalize().ok().map(|p| p.to_string_lossy().to_string());
                }
            }
        }

        None
    }

    fn compute_line_starts(content: &str) -> Vec<usize> {
        let mut starts = vec![0];
        for (i, byte) in content.bytes().enumerate() {
            if byte == b'\n' {
                starts.push(i + 1);
            }
        }
        starts
    }

    fn offset_to_line_col(offset: u32, line_starts: &[usize]) -> (u32, u32) {
        let offset = offset as usize;
        match line_starts.binary_search(&offset) {
            Ok(line_idx) => ((line_idx + 1) as u32, 1),
            Err(next_line_idx) => {
                let line_idx = next_line_idx - 1;
                let col = offset - line_starts[line_idx] + 1;
                ((line_idx + 1) as u32, col as u32)
            }
        }
    }

    /// Build the full dependency graph across all files in the repository
    pub fn build_dependency_graph(
        files: &mut [FileInfo],
        root_path: &Path,
        tsconfig: &TsConfigInfo,
    ) -> DependencyGraphResult {
        let parsed_results: Vec<(Vec<ImportDeclarationInfo>, Vec<String>)> = files
            .par_iter()
            .map(|file| {
                let p = Path::new(&file.path);
                Self::parse_file_imports(p, root_path, tsconfig)
            })
            .collect();

        for (file, (imports, exports)) in files.iter_mut().zip(parsed_results) {
            file.imports = imports;
            file.exported_symbols = exports;
        }

        let mut adjacency: HashMap<String, HashSet<String>> = HashMap::new();
        let mut reverse_adjacency: HashMap<String, HashSet<String>> = HashMap::new();
        let mut file_path_set: HashSet<String> = HashSet::new();

        for file in files.iter() {
            file_path_set.insert(file.path.clone());
            adjacency.entry(file.path.clone()).or_default();
            reverse_adjacency.entry(file.path.clone()).or_default();
        }

        let mut total_edges = 0;

        for file in files.iter() {
            for import_info in &file.imports {
                if let Some(ref target) = import_info.resolved_path {
                    if file_path_set.contains(target) && target != &file.path {
                        adjacency.get_mut(&file.path).unwrap().insert(target.clone());
                        reverse_adjacency.get_mut(target).unwrap().insert(file.path.clone());
                        total_edges += 1;
                    }
                }
            }
        }

        let circular_cycles = Self::detect_cycles(&adjacency);
        let circular_files: HashSet<String> = circular_cycles
            .iter()
            .flat_map(|c| c.files.iter().cloned())
            .collect();

        let mut nodes = Vec::new();
        let mut orphan_files = Vec::new();

        for file in files.iter() {
            let deps: Vec<String> = adjacency
                .get(&file.path)
                .map(|set| set.iter().cloned().collect())
                .unwrap_or_default();
            let dependents: Vec<String> = reverse_adjacency
                .get(&file.path)
                .map(|set| set.iter().cloned().collect())
                .unwrap_or_default();

            let fan_out = deps.len() as u32;
            let fan_in = dependents.len() as u32;
            let is_circular = circular_files.contains(&file.path);
            let is_orphan = fan_in == 0 && !file.framework_boundary.is_boundary;

            if is_orphan {
                orphan_files.push(file.relative_path.clone());
            }

            nodes.push(DependencyGraphNode {
                file_path: file.path.clone(),
                relative_path: file.relative_path.clone(),
                dependencies: deps,
                dependents,
                fan_in,
                fan_out,
                is_circular,
                is_orphan,
            });
        }

        DependencyGraphResult {
            total_nodes: nodes.len() as u32,
            total_edges,
            nodes,
            circular_cycles,
            orphan_files,
        }
    }

    fn detect_cycles(adjacency: &HashMap<String, HashSet<String>>) -> Vec<CircularCycle> {
        let mut cycles = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut path: Vec<String> = Vec::new();
        let mut path_set: HashSet<String> = HashSet::new();

        for start_node in adjacency.keys() {
            if !visited.contains(start_node) {
                Self::dfs_find_cycles(
                    start_node,
                    adjacency,
                    &mut visited,
                    &mut path,
                    &mut path_set,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn dfs_find_cycles(
        current: &str,
        adjacency: &HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
        path_set: &mut HashSet<String>,
        cycles: &mut Vec<CircularCycle>,
    ) {
        visited.insert(current.to_string());
        path.push(current.to_string());
        path_set.insert(current.to_string());

        if let Some(neighbors) = adjacency.get(current) {
            for neighbor in neighbors {
                if path_set.contains(neighbor) {
                    if let Some(cycle_start_idx) = path.iter().position(|p| p == neighbor) {
                        let cycle_files: Vec<String> = path[cycle_start_idx..].to_vec();
                        if cycle_files.len() >= 2 {
                            let cycle_len = cycle_files.len() as u32;
                            cycles.push(CircularCycle {
                                files: cycle_files,
                                cycle_length: cycle_len,
                            });
                        }
                    }
                } else if !visited.contains(neighbor) {
                    Self::dfs_find_cycles(neighbor, adjacency, visited, path, path_set, cycles);
                }
            }
        }

        path.pop();
        path_set.remove(current);
    }
}
