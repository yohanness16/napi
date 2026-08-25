use crate::scanner::SUPPORTED_EXTENSIONS;
use crate::types::{
    CloneCluster, CloneDetectionConfig, CloneDetectionResult, CodeCloneInstance,
};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPatternKind, ExportDefaultDeclarationKind, Expression, Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use rayon::prelude::*;
use siphasher::sip::SipHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::Hasher;
use std::path::Path;
use walkdir::WalkDir;

pub struct CloneDetector;

impl CloneDetector {
    /// Detect duplicated functions, methods, and logic clusters across the repository
    pub fn detect_clones(config: &CloneDetectionConfig) -> Result<CloneDetectionResult, String> {
        let root = Path::new(&config.root_path);
        if !root.exists() {
            return Err(format!("Root path does not exist: {}", config.root_path));
        }

        let canonical_root = root.canonicalize().map_err(|e| e.to_string())?;
        let min_lines = config.min_lines.unwrap_or(3);
        let min_nodes = config.min_ast_nodes.unwrap_or(10);

        let mut ignored = crate::scanner::DEFAULT_IGNORED_DIRS.to_vec();
        if let Some(ref custom_ign) = config.ignore_patterns {
            for ign in custom_ign {
                ignored.push(ign.as_str());
            }
        }

        // Collect all files
        let mut file_paths = Vec::new();
        for entry in WalkDir::new(&canonical_root).into_iter().filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !ignored.iter().any(|&ign| name == ign)
        }) {
            if let Ok(entry) = entry {
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                        if SUPPORTED_EXTENSIONS.contains(&ext) {
                            file_paths.push(entry.path().to_path_buf());
                        }
                    }
                }
            }
        }

        // Extract function instances from all files in parallel
        let all_instances: Vec<CodeCloneInstance> = file_paths
            .par_iter()
            .flat_map(|path| {
                Self::extract_file_functions(path, &canonical_root, min_lines, min_nodes)
            })
            .collect();

        // Group instances by structural hash
        let mut hash_groups: HashMap<String, Vec<CodeCloneInstance>> = HashMap::new();
        for instance in all_instances {
            let hash_key = Self::compute_instance_hash(&instance);
            hash_groups.entry(hash_key).or_default().push(instance);
        }

        // Filter groups with at least 2 occurrences
        let mut clusters = Vec::new();
        let mut total_clones = 0;
        let mut total_saved_lines = 0;
        let mut cluster_counter = 1;

        for (hash, instances) in hash_groups {
            if instances.len() >= 2 {
                let count = instances.len() as u32;
                let lines_per_inst = (instances[0].end_line - instances[0].start_line + 1) as u32;
                let potential_saved = (count - 1) * lines_per_inst;
                total_clones += count;
                total_saved_lines += potential_saved;

                let sample_fn_name = instances
                    .iter()
                    .find_map(|i| i.function_name.clone())
                    .unwrap_or_else(|| format!("utilityCluster{}", cluster_counter));

                let module_name = crate::planner::ArchitecturalPlanner::to_kebab_case(&sample_fn_name);
                let suggested_target = format!("src/shared/utils/{}.ts", module_name);

                clusters.push(CloneCluster {
                    cluster_id: format!("clone-cluster-{}", cluster_counter),
                    hash,
                    instance_count: count,
                    lines_per_instance: lines_per_inst,
                    potential_lines_saved: potential_saved,
                    ast_node_count: min_nodes,
                    instances,
                    suggested_module_name: sample_fn_name,
                    suggested_target_path: suggested_target,
                });

                cluster_counter += 1;
            }
        }

        // Sort clusters by highest potential lines saved
        clusters.sort_by(|a, b| b.potential_lines_saved.cmp(&a.potential_lines_saved));

        let total_clusters = clusters.len() as u32;

        Ok(CloneDetectionResult {
            total_clones_found: total_clones,
            total_clusters,
            total_lines_saved: total_saved_lines,
            clusters,
        })
    }

    /// Extract functions from a single file
    fn extract_file_functions(
        file_path: &Path,
        root_path: &Path,
        min_lines: u32,
        min_nodes: u32,
    ) -> Vec<CodeCloneInstance> {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let allocator = Allocator::default();
        let source_type = SourceType::from_path(file_path).unwrap_or_default();
        let ret = Parser::new(&allocator, &content, source_type).parse();

        let relative_path = pathdiff::diff_paths(file_path, root_path)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|| file_path.to_string_lossy().replace('\\', "/"));

        let line_starts = Self::compute_line_starts(&content);
        let mut instances = Vec::new();

        for stmt in &ret.program.body {
            Self::visit_statement(
                stmt,
                &content,
                file_path,
                &relative_path,
                &line_starts,
                min_lines,
                min_nodes,
                &mut instances,
            );
        }

        instances
    }

    fn visit_statement(
        stmt: &Statement,
        content: &str,
        file_path: &Path,
        relative_path: &str,
        line_starts: &[usize],
        min_lines: u32,
        min_nodes: u32,
        instances: &mut Vec<CodeCloneInstance>,
    ) {
        match stmt {
            Statement::FunctionDeclaration(func) => {
                let name = func.id.as_ref().map(|id| id.name.to_string());
                let span = func.span;
                Self::maybe_add_function_instance(
                    name,
                    span.start,
                    span.end,
                    content,
                    file_path,
                    relative_path,
                    line_starts,
                    min_lines,
                    instances,
                );
            }
            Statement::VariableDeclaration(var_decl) => {
                for decl in &var_decl.declarations {
                    if let Some(ref init) = decl.init {
                        let name = match &decl.id.kind {
                            BindingPatternKind::BindingIdentifier(id) => Some(id.name.to_string()),
                            _ => None,
                        };
                        match init {
                            Expression::ArrowFunctionExpression(arrow) => {
                                Self::maybe_add_function_instance(
                                    name,
                                    arrow.span.start,
                                    arrow.span.end,
                                    content,
                                    file_path,
                                    relative_path,
                                    line_starts,
                                    min_lines,
                                    instances,
                                );
                            }
                            Expression::FunctionExpression(func) => {
                                Self::maybe_add_function_instance(
                                    name,
                                    func.span.start,
                                    func.span.end,
                                    content,
                                    file_path,
                                    relative_path,
                                    line_starts,
                                    min_lines,
                                    instances,
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
            Statement::ExportNamedDeclaration(export_decl) => {
                if let Some(ref decl) = export_decl.declaration {
                    match decl {
                        oxc_ast::ast::Declaration::FunctionDeclaration(func) => {
                            let name = func.id.as_ref().map(|id| id.name.to_string());
                            let span = func.span;
                            Self::maybe_add_function_instance(
                                name,
                                span.start,
                                span.end,
                                content,
                                file_path,
                                relative_path,
                                line_starts,
                                min_lines,
                                instances,
                            );
                        }
                        oxc_ast::ast::Declaration::VariableDeclaration(var_decl) => {
                            for d in &var_decl.declarations {
                                if let Some(ref init) = d.init {
                                    let name = match &d.id.kind {
                                        BindingPatternKind::BindingIdentifier(id) => Some(id.name.to_string()),
                                        _ => None,
                                    };
                                    match init {
                                        Expression::ArrowFunctionExpression(arrow) => {
                                            Self::maybe_add_function_instance(
                                                name,
                                                arrow.span.start,
                                                arrow.span.end,
                                                content,
                                                file_path,
                                                relative_path,
                                                line_starts,
                                                min_lines,
                                                instances,
                                            );
                                        }
                                        Expression::FunctionExpression(func) => {
                                            Self::maybe_add_function_instance(
                                                name,
                                                func.span.start,
                                                func.span.end,
                                                content,
                                                file_path,
                                                relative_path,
                                                line_starts,
                                                min_lines,
                                                instances,
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Statement::ExportDefaultDeclaration(export_default) => {
                if let ExportDefaultDeclarationKind::FunctionDeclaration(func) = &export_default.declaration {
                    let name = func.id.as_ref().map(|id| id.name.to_string()).or(Some("defaultExport".to_string()));
                    Self::maybe_add_function_instance(
                        name,
                        func.span.start,
                        func.span.end,
                        content,
                        file_path,
                        relative_path,
                        line_starts,
                        min_lines,
                        instances,
                    );
                }
            }
            Statement::BlockStatement(block) => {
                for inner_stmt in &block.body {
                    Self::visit_statement(
                        inner_stmt,
                        content,
                        file_path,
                        relative_path,
                        line_starts,
                        min_lines,
                        min_nodes,
                        instances,
                    );
                }
            }
            _ => {}
        }
    }

    fn maybe_add_function_instance(
        function_name: Option<String>,
        start_byte: u32,
        end_byte: u32,
        content: &str,
        file_path: &Path,
        relative_path: &str,
        line_starts: &[usize],
        min_lines: u32,
        instances: &mut Vec<CodeCloneInstance>,
    ) {
        let (start_line, _) = Self::offset_to_line_col(start_byte, line_starts);
        let (end_line, _) = Self::offset_to_line_col(end_byte, line_starts);

        let lines_span = end_line.saturating_sub(start_line) + 1;
        if lines_span >= min_lines {
            let start = start_byte as usize;
            let end = (end_byte as usize).min(content.len());

            if start < end {
                let snippet = content[start..end].to_string();
                instances.push(CodeCloneInstance {
                    file_path: file_path.to_string_lossy().to_string(),
                    relative_path: relative_path.to_string(),
                    start_line,
                    end_line,
                    start_byte,
                    end_byte,
                    function_name,
                    code_snippet: snippet,
                });
            }
        }
    }

    /// Normalize code snippet tokens to create a structural representation
    fn normalize_tokens(code: &str) -> String {
        let mut normalized = String::with_capacity(code.len());
        let mut in_string = false;
        let mut string_char = '"';

        let chars: Vec<char> = code.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let ch = chars[i];

            if in_string {
                if ch == string_char && (i == 0 || chars[i - 1] != '\\') {
                    in_string = false;
                    normalized.push_str("\"$STR\"");
                }
                i += 1;
                continue;
            }

            if ch == '"' || ch == '\'' || ch == '`' {
                in_string = true;
                string_char = ch;
                i += 1;
                continue;
            }

            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '$') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                // Check if keyword
                if is_keyword(&word) {
                    normalized.push_str(&word);
                } else {
                    normalized.push_str("$VAR");
                }
                continue;
            }

            if !ch.is_whitespace() {
                normalized.push(ch);
            }

            i += 1;
        }

        normalized
    }

    /// Compute 64-bit SipHash for a normalized code snippet
    fn compute_instance_hash(instance: &CodeCloneInstance) -> String {
        let normalized = Self::normalize_tokens(&instance.code_snippet);
        let mut hasher = SipHasher::new();
        hasher.write(normalized.as_bytes());
        let hash_val = hasher.finish();
        format!("{:016x}", hash_val)
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
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "const"
            | "let"
            | "var"
            | "function"
            | "return"
            | "if"
            | "else"
            | "for"
            | "while"
            | "do"
            | "switch"
            | "case"
            | "default"
            | "break"
            | "continue"
            | "try"
            | "catch"
            | "finally"
            | "throw"
            | "new"
            | "typeof"
            | "instanceof"
            | "async"
            | "await"
            | "yield"
            | "import"
            | "export"
            | "from"
            | "class"
            | "extends"
            | "super"
            | "this"
            | "null"
            | "undefined"
            | "true"
            | "false"
    )
}
