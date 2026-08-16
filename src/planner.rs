use crate::types::{
    ArchitectureTarget, FileInfo, FileMoveAction, FrameworkType, NamingConvention, PatchAction,
    PlanConfig, RefactorPlan, RefactorSummary, TsConfigInfo,
};
use std::collections::HashMap;
use std::path::Path;

pub struct ArchitecturalPlanner;

impl ArchitecturalPlanner {
    /// Generate a complete refactoring plan
    pub fn plan(
        config: &PlanConfig,
        files: &[FileInfo],
        framework: FrameworkType,
        tsconfig: &TsConfigInfo,
    ) -> Result<RefactorPlan, String> {
        let root = Path::new(&config.root_path);
        let canonical_root = root.canonicalize().map_err(|e| e.to_string())?;

        let mut file_moves = Vec::new();
        let mut protected_files = Vec::new();
        let mut old_to_new_path_map: HashMap<String, String> = HashMap::new();

        for file in files {
            // Check if file is a protected framework route or root file
            if file.framework_boundary.is_protected_route {
                protected_files.push(file.relative_path.clone());
                old_to_new_path_map.insert(file.path.clone(), file.path.clone());
                continue;
            }

            // Determine target new relative path based on architecture and naming
            let new_relative_path = Self::compute_target_path(
                &file.relative_path,
                &file.file_name,
                &file.extension,
                config.target_architecture,
                config.naming_convention,
                config.custom_feature_mappings.as_ref(),
                framework,
            );

            let new_full_path = canonical_root.join(&new_relative_path).to_string_lossy().to_string();

            if new_full_path != file.path {
                file_moves.push(FileMoveAction {
                    original_path: file.path.clone(),
                    original_relative_path: file.relative_path.clone(),
                    new_path: new_full_path.clone(),
                    new_relative_path: new_relative_path.clone(),
                    reason: format!(
                        "Migrate to {:?} architecture with {:?} naming",
                        config.target_architecture, config.naming_convention
                    ),
                    is_protected_framework_file: false,
                });
            }

            old_to_new_path_map.insert(file.path.clone(), new_full_path);
        }

        // Now compute all required AST import/export byte patches
        let mut patches = Vec::new();

        for file in files {
            let current_old_path = &file.path;
            let current_new_path = old_to_new_path_map.get(current_old_path).unwrap_or(current_old_path);
            let current_new_dir = Path::new(current_new_path).parent().unwrap_or(&canonical_root);

            for import_info in &file.imports {
                // If it resolves to an internal file in the repository
                if let Some(ref target_old_path) = import_info.resolved_path {
                    if let Some(target_new_path) = old_to_new_path_map.get(target_old_path) {
                        let needs_update = current_new_path != current_old_path
                            || target_new_path != target_old_path;

                        if needs_update {
                            let replacement = Self::calculate_new_import_specifier(
                                &import_info.specifier,
                                current_new_dir,
                                Path::new(target_new_path),
                                &canonical_root,
                                tsconfig,
                            );

                            if replacement != import_info.specifier {
                                patches.push(PatchAction {
                                    file_path: file.path.clone(),
                                    span_start: import_info.span.start,
                                    span_end: import_info.span.end,
                                    original_specifier: import_info.specifier.clone(),
                                    replacement_specifier: replacement,
                                    reason: format!(
                                        "Rewire import from moved target: `{}`",
                                        Path::new(target_new_path).file_name().unwrap_or_default().to_string_lossy()
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }

        let summary = RefactorSummary {
            total_files_moved: file_moves.len() as u32,
            total_imports_patched: patches.len() as u32,
            total_protected_files: protected_files.len() as u32,
            target_architecture: config.target_architecture,
            naming_convention: config.naming_convention,
        };

        Ok(RefactorPlan {
            root_path: canonical_root.to_string_lossy().to_string(),
            target_architecture: config.target_architecture,
            naming_convention: config.naming_convention,
            file_moves,
            patches,
            protected_files,
            summary,
        })
    }

    /// Calculate the target relative path for a file according to architecture and naming rules
    fn compute_target_path(
        relative_path_str: &str,
        file_name: &str,
        extension: &str,
        target_arch: ArchitectureTarget,
        naming: NamingConvention,
        custom_mappings: Option<&HashMap<String, String>>,
        _framework: FrameworkType,
    ) -> String {
        let normalized = relative_path_str.replace('\\', "/");
        let stem = Path::new(file_name).file_stem().and_then(|s| s.to_str()).unwrap_or(file_name);

        // Normalize filename according to naming convention
        let normalized_stem = Self::apply_naming_convention(stem, naming);
        let normalized_filename = format!("{}.{}", normalized_stem, extension);

        // Check custom feature mappings first
        if let Some(mappings) = custom_mappings {
            for (key, target_dir) in mappings {
                if normalized.contains(key) {
                    return format!("{}/{}", target_dir.trim_end_matches('/'), normalized_filename);
                }
            }
        }

        // Framework entrypoints/root files stay in place
        if normalized == "package.json"
            || normalized == "tsconfig.json"
            || normalized == "src/index.ts"
            || normalized == "src/main.ts"
            || normalized == "src/App.tsx"
            || normalized == "index.ts"
            || normalized == "main.ts"
        {
            return normalized;
        }

        // Categorize file role based on path and file name
        let role = Self::classify_file_role(&normalized, stem);
        let feature = Self::extract_feature_name(&normalized);

        match target_arch {
            ArchitectureTarget::FeatureBased => {
                if feature == "shared" || feature == "common" {
                    format!("src/shared/{}/{}", role, normalized_filename)
                } else {
                    format!("src/features/{}/{}/{}", feature, role, normalized_filename)
                }
            }
            ArchitectureTarget::DomainDrivenDesign => {
                match role.as_str() {
                    "types" | "models" | "entities" => {
                        format!("src/domain/{}/models/{}", feature, normalized_filename)
                    }
                    "services" | "usecases" | "api" => {
                        format!("src/application/{}/services/{}", feature, normalized_filename)
                    }
                    "utils" | "helpers" | "infrastructure" => {
                        format!("src/infrastructure/shared/{}", normalized_filename)
                    }
                    "components" | "hooks" => {
                        format!("src/presentation/{}/components/{}", feature, normalized_filename)
                    }
                    _ => {
                        format!("src/domain/{}/{}/{}", feature, role, normalized_filename)
                    }
                }
            }
            ArchitectureTarget::Layered => {
                format!("src/{}/{}", role, normalized_filename)
            }
            ArchitectureTarget::Custom => normalized,
        }
    }

    /// Classify the architectural role of a file (components, hooks, services, utils, types)
    fn classify_file_role(path_str: &str, stem: &str) -> String {
        let lower = path_str.to_lowercase();
        let stem_lower = stem.to_lowercase();

        if lower.contains("component") || stem_lower.ends_with("button") || stem_lower.ends_with("card") || stem_lower.ends_with("modal") || stem_lower.ends_with("form") || stem_lower.ends_with("view") || stem_lower.ends_with("header") || stem_lower.ends_with("footer") {
            "components".to_string()
        } else if lower.contains("hook") || stem_lower.starts_with("use") {
            "hooks".to_string()
        } else if lower.contains("service") || stem_lower.ends_with("service") || lower.contains("api") || stem_lower.ends_with("client") {
            "services".to_string()
        } else if lower.contains("util") || lower.contains("helper") || stem_lower.ends_with("utils") || stem_lower.ends_with("helper") {
            "utils".to_string()
        } else if lower.contains("type") || lower.contains("model") || lower.contains("interface") || stem_lower.ends_with("types") || stem_lower.ends_with("dto") {
            "types".to_string()
        } else if lower.contains("constant") || stem_lower.ends_with("constants") {
            "constants".to_string()
        } else {
            "modules".to_string()
        }
    }

    /// Extract domain or feature name from path segments
    fn extract_feature_name(path_str: &str) -> String {
        let segments: Vec<&str> = path_str.split('/').collect();
        let common_keywords = [
            "src", "lib", "app", "components", "hooks", "services", "utils",
            "types", "models", "features", "presentation", "domain", "application",
            "infrastructure", "shared", "common", "ui",
        ];

        for seg in &segments {
            let s = seg.to_lowercase();
            if !s.is_empty() && !common_keywords.contains(&s.as_str()) && !s.ends_with(".ts") && !s.ends_with(".tsx") && !s.ends_with(".js") && !s.ends_with(".jsx") {
                return Self::to_kebab_case(seg);
            }
        }

        "shared".to_string()
    }

    /// Transform an identifier or filename to the target naming convention
    pub fn apply_naming_convention(name: &str, convention: NamingConvention) -> String {
        match convention {
            NamingConvention::KebabCase => Self::to_kebab_case(name),
            NamingConvention::PascalCase => Self::to_pascal_case(name),
            NamingConvention::CamelCase => Self::to_camel_case(name),
            NamingConvention::SnakeCase => Self::to_snake_case(name),
            NamingConvention::Preserve => name.to_string(),
        }
    }

    pub fn to_kebab_case(s: &str) -> String {
        let mut result = String::new();
        let mut prev_is_upper = false;
        let mut prev_is_sep = true;

        for ch in s.chars() {
            if ch == '-' || ch == '_' || ch == '.' || ch == ' ' {
                if !prev_is_sep && !result.is_empty() {
                    result.push('-');
                    prev_is_sep = true;
                }
                prev_is_upper = false;
            } else if ch.is_uppercase() {
                if !prev_is_upper && !prev_is_sep && !result.is_empty() {
                    result.push('-');
                }
                result.push(ch.to_ascii_lowercase());
                prev_is_upper = true;
                prev_is_sep = false;
            } else {
                result.push(ch);
                prev_is_upper = false;
                prev_is_sep = false;
            }
        }

        result.trim_matches('-').to_string()
    }

    pub fn to_pascal_case(s: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;

        for ch in s.chars() {
            if ch == '-' || ch == '_' || ch == '.' || ch == ' ' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(ch.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }

        result
    }

    pub fn to_camel_case(s: &str) -> String {
        let pascal = Self::to_pascal_case(s);
        let mut chars = pascal.chars();
        match chars.next() {
            Some(first) => format!("{}{}", first.to_ascii_lowercase(), chars.as_str()),
            None => String::new(),
        }
    }

    pub fn to_snake_case(s: &str) -> String {
        Self::to_kebab_case(s).replace('-', "_")
    }

    /// Calculate the new relative import specifier from the new importer directory to the new target file
    pub fn calculate_new_import_specifier(
        original_specifier: &str,
        new_importer_dir: &Path,
        new_target_path: &Path,
        root_path: &Path,
        tsconfig: &TsConfigInfo,
    ) -> String {
        // If original was a tsconfig alias (e.g. "@/components/Button"), check if we should keep/update alias
        if original_specifier.starts_with('@') || original_specifier.starts_with('~') {
            for (alias_pattern, target_patterns) in &tsconfig.paths {
                let prefix = alias_pattern.trim_end_matches('*');
                if original_specifier.starts_with(prefix) {
                    for target_pattern in target_patterns {
                        let target_prefix = target_pattern.trim_end_matches('*');
                        let base_dir = if let Some(ref base) = tsconfig.base_url {
                            root_path.join(base)
                        } else {
                            root_path.to_path_buf()
                        };

                        let mapped_base = base_dir.join(target_prefix);
                        if let Ok(rel_to_alias) = new_target_path.strip_prefix(&mapped_base) {
                            let rel_str = rel_to_alias.to_string_lossy().replace('\\', "/");
                            let without_ext = rel_str.trim_end_matches(".tsx")
                                .trim_end_matches(".ts")
                                .trim_end_matches(".jsx")
                                .trim_end_matches(".js");
                            let without_index = if without_ext.ends_with("/index") {
                                &without_ext[..without_ext.len() - 6]
                            } else {
                                without_ext
                            };
                            return format!("{}{}", prefix, without_index);
                        }
                    }
                }
            }
        }

        // Standard relative path calculus
        let relative = match pathdiff::diff_paths(new_target_path, new_importer_dir) {
            Some(r) => r,
            None => return original_specifier.to_string(),
        };

        let mut rel_str = relative.to_string_lossy().replace('\\', "/");

        // Strip file extensions (.ts, .tsx, .js, .jsx)
        if rel_str.ends_with(".tsx") {
            rel_str = rel_str[..rel_str.len() - 4].to_string();
        } else if rel_str.ends_with(".jsx") {
            rel_str = rel_str[..rel_str.len() - 4].to_string();
        } else if rel_str.ends_with(".ts") {
            rel_str = rel_str[..rel_str.len() - 3].to_string();
        } else if rel_str.ends_with(".js") {
            rel_str = rel_str[..rel_str.len() - 3].to_string();
        }

        // Strip /index if target is index file
        if rel_str.ends_with("/index") {
            rel_str = rel_str[..rel_str.len() - 6].to_string();
        }

        if !rel_str.starts_with('.') {
            format!("./{}", rel_str)
        } else {
            rel_str
        }
    }
}
