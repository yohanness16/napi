use crate::types::{
    FileInfo, FrameworkBoundaryInfo, FrameworkType, RepositoryScanResult, ScanConfig, TsConfigInfo,
};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Standard directories to ignore during repository scanning
pub const DEFAULT_IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "build",
    ".next",
    ".remix",
    ".turbo",
    ".cache",
    "coverage",
    "target",
    "out",
    ".nuxt",
    ".output",
    "npm",
];

/// File extensions supported by the AST parser
pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "ts", "tsx", "js", "jsx", "mjs", "cjs", "mts", "cts",
];

pub struct Scanner;

impl Scanner {
    /// Detect the primary framework of the repository
    pub fn detect_framework(root: &Path) -> (FrameworkType, String) {
        let has_next_app = root.join("app").is_dir() || root.join("src").join("app").is_dir();
        let has_next_pages = root.join("pages").is_dir() || root.join("src").join("pages").is_dir();
        let has_remix_routes = root.join("app").join("routes").is_dir() || root.join("src").join("app").join("routes").is_dir();
        let has_nest_cli = root.join("nest-cli.json").exists();
        let has_vite = root.join("vite.config.ts").exists() || root.join("vite.config.js").exists() || root.join("vite.config.mjs").exists();
        let has_nuxt = root.join("nuxt.config.ts").exists() || root.join("nuxt.config.js").exists();

        // Check package.json dependencies if available
        let package_json_path = root.join("package.json");
        let package_deps = if package_json_path.exists() {
            fs::read_to_string(&package_json_path).unwrap_or_default()
        } else {
            String::new()
        };

        if has_next_app && package_deps.contains("\"next\"") {
            (
                FrameworkType::NextAppRouter,
                "Next.js App Router (app/ layout and server component routing conventions)".to_string(),
            )
        } else if has_next_pages && package_deps.contains("\"next\"") {
            (
                FrameworkType::NextPagesRouter,
                "Next.js Pages Router (pages/ directory based routing)".to_string(),
            )
        } else if (has_remix_routes || package_deps.contains("\"@remix-run/\"")) && package_deps.contains("\"@remix-run/react\"") {
            (
                FrameworkType::Remix,
                "Remix / React Router v7 (nested file-based route conventions)".to_string(),
            )
        } else if has_nest_cli || package_deps.contains("\"@nestjs/core\"") {
            (
                FrameworkType::NestJs,
                "NestJS (modular dependency-injected enterprise TypeScript backend)".to_string(),
            )
        } else if has_nuxt || package_deps.contains("\"nuxt\"") {
            (
                FrameworkType::Vue,
                "Nuxt / Vue (Vue SFC file-based framework)".to_string(),
            )
        } else if has_vite && package_deps.contains("\"react\"") {
            (
                FrameworkType::Vite,
                "Vite + React (Fast ESM build tooling)".to_string(),
            )
        } else if package_deps.contains("\"express\"") || package_deps.contains("\"fastify\"") {
            (
                FrameworkType::Express,
                "Express / Fastify Node.js HTTP Server".to_string(),
            )
        } else if package_deps.contains("\"react\"") {
            (
                FrameworkType::React,
                "React SPA / Library".to_string(),
            )
        } else {
            (
                FrameworkType::Generic,
                "Generic TypeScript / JavaScript repository".to_string(),
            )
        }
    }

    /// Parse tsconfig.json or jsconfig.json to discover base_url and path mappings
    pub fn parse_tsconfig(root: &Path, custom_path: Option<&str>) -> TsConfigInfo {
        let tsconfig_file = if let Some(p) = custom_path {
            root.join(p)
        } else if root.join("tsconfig.json").exists() {
            root.join("tsconfig.json")
        } else if root.join("jsconfig.json").exists() {
            root.join("jsconfig.json")
        } else {
            return TsConfigInfo::default();
        };

        if !tsconfig_file.exists() {
            return TsConfigInfo::default();
        }

        let content = match fs::read_to_string(&tsconfig_file) {
            Ok(c) => c,
            Err(_) => return TsConfigInfo::default(),
        };

        // Strip comments from tsconfig JSON (single-line // and block /* */)
        let cleaned_json = Self::strip_json_comments(&content);

        let parsed: serde_json::Value = match serde_json::from_str(&cleaned_json) {
            Ok(v) => v,
            Err(_) => return TsConfigInfo::default(),
        };

        let compiler_options = parsed.get("compilerOptions");
        if let Some(opts) = compiler_options {
            let base_url = opts.get("baseUrl").and_then(|v| v.as_str()).map(|s| s.to_string());
            let mut paths: HashMap<String, Vec<String>> = HashMap::new();

            if let Some(paths_obj) = opts.get("paths").and_then(|v| v.as_object()) {
                for (alias, targets) in paths_obj {
                    if let Some(target_arr) = targets.as_array() {
                        let target_strings: Vec<String> = target_arr
                            .iter()
                            .filter_map(|t| t.as_str().map(|s| s.to_string()))
                            .collect();
                        paths.insert(alias.clone(), target_strings);
                    }
                }
            }

            TsConfigInfo { base_url, paths }
        } else {
            TsConfigInfo::default()
        }
    }

    /// Helper to strip JSON comments (// and /* */) for standard tsconfig files
    fn strip_json_comments(json_str: &str) -> String {
        let mut result = String::with_capacity(json_str.len());
        let mut in_string = false;
        let mut in_single_comment = false;
        let mut in_multi_comment = false;
        let chars: Vec<char> = json_str.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            let ch = chars[i];
            let next_ch = if i + 1 < len { Some(chars[i + 1]) } else { None };

            if in_single_comment {
                if ch == '\n' {
                    in_single_comment = false;
                    result.push(ch);
                }
                i += 1;
                continue;
            }

            if in_multi_comment {
                if ch == '*' && next_ch == Some('/') {
                    in_multi_comment = false;
                    i += 2;
                    continue;
                }
                i += 1;
                continue;
            }

            if !in_string && ch == '/' && next_ch == Some('/') {
                in_single_comment = true;
                i += 2;
                continue;
            }

            if !in_string && ch == '/' && next_ch == Some('*') {
                in_multi_comment = true;
                i += 2;
                continue;
            }

            if ch == '"' && (i == 0 || chars[i - 1] != '\\') {
                in_string = !in_string;
            }

            result.push(ch);
            i += 1;
        }

        result
    }

    /// Detect framework boundary rules and protected status for a given file path
    pub fn evaluate_framework_boundary(
        relative_path_str: &str,
        file_content: &str,
        framework: FrameworkType,
    ) -> FrameworkBoundaryInfo {
        let path = Path::new(relative_path_str);
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or_default();
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();

        let normalized_path = relative_path_str.replace('\\', "/");

        // Check for directives like "use client" or "use server"
        let directive = if file_content.starts_with("\"use client\"")
            || file_content.starts_with("'use client'")
            || file_content.contains("\n\"use client\"")
            || file_content.contains("\n'use client'")
        {
            Some("use client".to_string())
        } else if file_content.starts_with("\"use server\"")
            || file_content.starts_with("'use server'")
            || file_content.contains("\n\"use server\"")
            || file_content.contains("\n'use server'")
        {
            Some("use server".to_string())
        } else {
            None
        };

        // Next.js App Router rules
        if framework == FrameworkType::NextAppRouter || normalized_path.starts_with("app/") || normalized_path.starts_with("src/app/") {
            let next_special_files = [
                "layout", "page", "loading", "not-found", "error", "global-error",
                "route", "template", "default", "opengraph-image", "twitter-image",
                "sitemap", "robots", "manifest", "icon", "apple-icon",
            ];

            if next_special_files.contains(&stem) {
                return FrameworkBoundaryInfo {
                    is_boundary: true,
                    is_protected_route: true,
                    boundary_type: format!("NextAppRouter:{}", stem),
                    description: format!("Next.js App Router convention file `{}` (protected route hierarchy)", file_name),
                    directive,
                };
            }

            // Route groups: (group) or Parallel routes: @slot or Intercepting routes: (.)
            if normalized_path.contains("/(") || normalized_path.contains("/@") || normalized_path.contains("/(.)") {
                return FrameworkBoundaryInfo {
                    is_boundary: true,
                    is_protected_route: true,
                    boundary_type: "NextAppRouter:SpecialRoute".to_string(),
                    description: "Next.js Route Group, Parallel Route or Intercepting Route segment".to_string(),
                    directive,
                };
            }
        }

        // Next.js Pages Router rules
        if framework == FrameworkType::NextPagesRouter || normalized_path.starts_with("pages/") || normalized_path.starts_with("src/pages/") {
            if stem == "_app" || stem == "_document" || stem == "_error" || stem == "404" || stem == "500" {
                return FrameworkBoundaryInfo {
                    is_boundary: true,
                    is_protected_route: true,
                    boundary_type: format!("NextPagesRouter:{}", stem),
                    description: format!("Next.js Pages Router root convention `{}`", file_name),
                    directive,
                };
            }

            if normalized_path.contains("/api/") {
                return FrameworkBoundaryInfo {
                    is_boundary: true,
                    is_protected_route: true,
                    boundary_type: "NextPagesRouter:ApiRoute".to_string(),
                    description: "Next.js API route handler".to_string(),
                    directive,
                };
            }

            return FrameworkBoundaryInfo {
                is_boundary: true,
                is_protected_route: true,
                boundary_type: "NextPagesRouter:Page".to_string(),
                description: format!("Next.js Page route `{}`", relative_path_str),
                directive,
            };
        }

        // Remix / React Router v7 rules
        if framework == FrameworkType::Remix || normalized_path.starts_with("app/routes/") || normalized_path.starts_with("src/app/routes/") {
            if stem == "root" || stem == "entry.client" || stem == "entry.server" {
                return FrameworkBoundaryInfo {
                    is_boundary: true,
                    is_protected_route: true,
                    boundary_type: format!("Remix:{}", stem),
                    description: format!("Remix framework entry `{}`", file_name),
                    directive,
                };
            }

            if normalized_path.contains("/routes/") {
                return FrameworkBoundaryInfo {
                    is_boundary: true,
                    is_protected_route: true,
                    boundary_type: "Remix:Route".to_string(),
                    description: format!("Remix nested route `{}`", relative_path_str),
                    directive,
                };
            }
        }

        // General entry points
        if normalized_path == "src/index.ts"
            || normalized_path == "src/main.ts"
            || normalized_path == "src/server.ts"
            || normalized_path == "src/App.tsx"
            || normalized_path == "index.ts"
            || normalized_path == "server.ts"
        {
            return FrameworkBoundaryInfo {
                is_boundary: true,
                is_protected_route: false,
                boundary_type: "AppEntryPoint".to_string(),
                description: format!("Application root entrypoint `{}`", file_name),
                directive,
            };
        }

        FrameworkBoundaryInfo {
            is_boundary: false,
            is_protected_route: false,
            boundary_type: "StandardModule".to_string(),
            description: "Refactorable source module".to_string(),
            directive,
        }
    }

    /// Perform a full scan of the repository
    pub fn scan(config: &ScanConfig) -> Result<RepositoryScanResult, String> {
        let root = Path::new(&config.root_path);
        if !root.exists() {
            return Err(format!("Repository root path does not exist: {}", config.root_path));
        }

        let canonical_root = root.canonicalize().map_err(|e| e.to_string())?;
        let (framework, framework_desc) = Self::detect_framework(&canonical_root);
        let tsconfig = Self::parse_tsconfig(&canonical_root, config.tsconfig_path.as_deref());

        let mut ignored_dirs = DEFAULT_IGNORED_DIRS.to_vec();
        if let Some(ref custom_ignores) = config.ignore_patterns {
            for ign in custom_ignores {
                ignored_dirs.push(ign.as_str());
            }
        }

        // Collect all candidate files
        let mut file_paths = Vec::new();
        for entry in WalkDir::new(&canonical_root).follow_links(false).into_iter().filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !ignored_dirs.iter().any(|&ign| name == ign)
        }) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                    if SUPPORTED_EXTENSIONS.contains(&ext) {
                        file_paths.push(entry.path().to_path_buf());
                    }
                }
            }
        }

        // Process files in parallel with rayon
        let file_infos: Vec<FileInfo> = file_paths
            .par_iter()
            .filter_map(|path| {
                let relative = match pathdiff::diff_paths(path, &canonical_root) {
                    Some(r) => r.to_string_lossy().to_string(),
                    None => path.to_string_lossy().to_string(),
                };

                let content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(_) => return None,
                };

                let line_count = content.lines().count() as u32;
                let size_bytes = content.len() as u32;
                let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let extension = path.extension().unwrap_or_default().to_string_lossy().to_string();

                let framework_boundary = Self::evaluate_framework_boundary(&relative, &content, framework);

                Some(FileInfo {
                    path: path.to_string_lossy().to_string(),
                    relative_path: relative,
                    file_name,
                    extension,
                    size_bytes,
                    line_count,
                    framework_boundary,
                    imports: Vec::new(),
                    exported_symbols: Vec::new(),
                })
            })
            .collect();

        let total_files = file_infos.len() as u32;
        let total_lines: u32 = file_infos.iter().map(|f| f.line_count).sum();

        Ok(RepositoryScanResult {
            root_path: canonical_root.to_string_lossy().to_string(),
            framework,
            framework_description: framework_desc,
            total_files,
            total_lines,
            files: file_infos,
            dependency_graph: crate::types::DependencyGraphResult {
                total_nodes: 0,
                total_edges: 0,
                nodes: Vec::new(),
                circular_cycles: Vec::new(),
                orphan_files: Vec::new(),
            },
            tsconfig,
        })
    }
}
