use crate::patcher::BytePatcher;
use crate::types::{
    ApplyOptions, ApplyResult, DiffPreviewResult, GitStatusResult, JournalEntry,
    JournalStatus, PatchAction, RefactorPlan, RollbackResult,
};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const JOURNAL_FILENAME: &str = ".refactor-journal.json";

pub struct TransactionEngine;

impl TransactionEngine {
    /// Inspect Git status for guardrails and cleanliness verification
    pub fn get_git_status(root_path: &str) -> GitStatusResult {
        let root = Path::new(root_path);

        // Check if git is available and if this is a git repo
        let git_dir_check = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(root)
            .output();

        let (is_git_repo, git_root) = match git_dir_check {
            Ok(output) if output.status.success() => {
                let toplevel = Command::new("git")
                    .args(["rev-parse", "--show-toplevel"])
                    .current_dir(root)
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string());
                (true, toplevel)
            }
            _ => (false, None),
        };

        if !is_git_repo {
            return GitStatusResult {
                is_git_repo: false,
                is_clean: true,
                git_root: None,
                branch: None,
                modified_files: Vec::new(),
                untracked_files: Vec::new(),
                warning: Some("Directory is not a Git repository. Recommend initializing Git for version control safety.".to_string()),
            };
        }

        // Get branch
        let branch = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(root)
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string());

        // Check status
        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(root)
            .output();

        let mut modified_files = Vec::new();
        let mut untracked_files = Vec::new();

        if let Ok(output) = status_output {
            let status_str = String::from_utf8_lossy(&output.stdout);
            for line in status_str.lines() {
                if line.len() > 3 {
                    let code = &line[0..2];
                    let file = line[3..].trim();
                    if code.contains('?') {
                        untracked_files.push(file.to_string());
                    } else {
                        modified_files.push(file.to_string());
                    }
                }
            }
        }

        let is_clean = modified_files.is_empty();
        let warning = if !is_clean {
            Some(format!(
                "Working tree has {} uncommitted changes. Use --force to proceed anyway.",
                modified_files.len()
            ))
        } else {
            None
        };

        GitStatusResult {
            is_git_repo: true,
            is_clean,
            git_root,
            branch,
            modified_files,
            untracked_files,
            warning,
        }
    }

    /// Generate unified diff preview for the entire refactor plan
    pub fn preview_diff(plan: &RefactorPlan) -> Result<DiffPreviewResult, String> {
        let root = Path::new(&plan.root_path);

        // Group patches by file path
        let mut patches_by_file: HashMap<String, Vec<PatchAction>> = HashMap::new();
        for patch in &plan.patches {
            patches_by_file.entry(patch.file_path.clone()).or_default().push(patch.clone());
        }

        let mut move_map: HashMap<String, String> = HashMap::new();
        for m in &plan.file_moves {
            move_map.insert(m.original_path.clone(), m.new_path.clone());
        }

        let mut all_affected_files: HashSet<String> = HashSet::new();
        for m in &plan.file_moves {
            all_affected_files.insert(m.original_path.clone());
        }
        for (f, _) in &patches_by_file {
            all_affected_files.insert(f.clone());
        }

        let mut file_diffs = Vec::new();
        let mut total_additions = 0;
        let mut total_deletions = 0;

        for file_path in all_affected_files {
            let p = Path::new(&file_path);
            let original_content = match fs::read_to_string(p) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let patches = patches_by_file.get(&file_path).map(|v| v.as_slice()).unwrap_or(&[]);
            let patched_content = BytePatcher::patch_content(&original_content, patches)?;

            let is_moved = move_map.contains_key(&file_path);
            let new_path = move_map.get(&file_path).cloned();

            let rel_path = pathdiff::diff_paths(p, root)
                .map(|r| r.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|| file_path.clone());

            let diff = BytePatcher::generate_unified_diff(
                &original_content,
                &patched_content,
                &file_path,
                &rel_path,
                false,
                false,
                is_moved,
                Some(file_path.clone()),
                new_path,
            );

            total_additions += diff.additions;
            total_deletions += diff.deletions;
            file_diffs.push(diff);
        }

        let total_files = file_diffs.len() as u32;

        Ok(DiffPreviewResult {
            total_files_changed: total_files,
            total_additions,
            total_deletions,
            file_diffs,
        })
    }

    /// Execute atomic refactoring transaction and write journal
    pub fn apply(plan: &RefactorPlan, options: &ApplyOptions) -> Result<ApplyResult, String> {
        let dry_run = options.dry_run.unwrap_or(false);
        let force = options.force.unwrap_or(false);
        let skip_git = options.skip_git_check.unwrap_or(false);

        // Safety check git status
        if !skip_git && !force {
            let git_status = Self::get_git_status(&plan.root_path);
            if git_status.is_git_repo && !git_status.is_clean {
                return Err(format!(
                    "Git repository has {} modified files. Commit changes or use `--force` to bypass.",
                    git_status.modified_files.len()
                ));
            }
        }

        let tx_id = format!("tx-{}", Utc::now().timestamp_millis());

        if dry_run {
            return Ok(ApplyResult {
                success: true,
                files_moved: plan.file_moves.len() as u32,
                files_patched: plan.patches.len() as u32,
                journal_path: None,
                transaction_id: tx_id,
                message: "Dry run completed successfully. No files were modified on disk.".to_string(),
            });
        }

        // Group patches by file
        let mut patches_by_file: HashMap<String, Vec<PatchAction>> = HashMap::new();
        for patch in &plan.patches {
            patches_by_file.entry(patch.file_path.clone()).or_default().push(patch.clone());
        }

        let mut move_map: HashMap<String, String> = HashMap::new();
        for m in &plan.file_moves {
            move_map.insert(m.original_path.clone(), m.new_path.clone());
        }

        let mut original_files_backup: HashMap<String, String> = HashMap::new();
        let mut modified_contents: HashMap<String, String> = HashMap::new();

        // 1. Prepare and validate all patched contents before touching disk
        let mut all_affected_files: HashSet<String> = HashSet::new();
        for m in &plan.file_moves {
            all_affected_files.insert(m.original_path.clone());
        }
        for (f, _) in &patches_by_file {
            all_affected_files.insert(f.clone());
        }

        for file_path in &all_affected_files {
            let p = Path::new(file_path);
            let original_content = fs::read_to_string(p)
                .map_err(|e| format!("Failed to read source file `{}`: {}", file_path, e))?;

            original_files_backup.insert(file_path.clone(), original_content.clone());

            let patches = patches_by_file.get(file_path).map(|v| v.as_slice()).unwrap_or(&[]);
            let patched_content = BytePatcher::patch_content(&original_content, patches)?;

            // Verify syntax of patched code
            BytePatcher::verify_syntax(&patched_content, p)?;

            modified_contents.insert(file_path.clone(), patched_content);
        }

        // 2. Perform file write transactions
        let mut created_files = Vec::new();
        let mut created_dirs = Vec::new();

        for file_path in &all_affected_files {
            let patched_content = modified_contents.get(file_path).unwrap();
            let target_path_str = move_map.get(file_path).unwrap_or(file_path);
            let target_path = Path::new(target_path_str);

            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory `{}`: {}", parent.display(), e))?;
                    created_dirs.push(parent.to_string_lossy().to_string());
                }
            }

            fs::write(target_path, patched_content)
                .map_err(|e| format!("Failed to write target file `{}`: {}", target_path.display(), e))?;
            created_files.push(target_path_str.clone());

            // If file was moved, remove original file if it is distinct
            if move_map.contains_key(file_path) && file_path != target_path_str {
                let _ = fs::remove_file(Path::new(file_path));
            }
        }

        // 3. Write transaction journal
        let journal_dir = options.journal_dir.as_deref().unwrap_or(&plan.root_path);
        let journal_path = Path::new(journal_dir).join(JOURNAL_FILENAME);

        let journal_entry = JournalEntry {
            transaction_id: tx_id.clone(),
            timestamp: Utc::now().to_rfc3339(),
            root_path: plan.root_path.clone(),
            target_architecture: plan.target_architecture,
            naming_convention: plan.naming_convention,
            file_moves: plan.file_moves.clone(),
            patches: plan.patches.clone(),
            original_files_backup,
            created_files,
            created_directories: created_dirs,
            status: JournalStatus::Applied,
        };

        let journal_json = serde_json::to_string_pretty(&journal_entry)
            .map_err(|e| format!("Failed to serialize transaction journal: {}", e))?;

        fs::write(&journal_path, journal_json)
            .map_err(|e| format!("Failed to write journal file `{}`: {}", journal_path.display(), e))?;

        Ok(ApplyResult {
            success: true,
            files_moved: plan.file_moves.len() as u32,
            files_patched: plan.patches.len() as u32,
            journal_path: Some(journal_path.to_string_lossy().to_string()),
            transaction_id: tx_id,
            message: format!(
                "Successfully applied refactor: {} files moved, {} imports rewired.",
                plan.file_moves.len(),
                plan.patches.len()
            ),
        })
    }

    /// Roll back a previously applied refactor using the journal
    pub fn rollback(journal_path_opt: Option<&str>, root_path_opt: Option<&str>) -> Result<RollbackResult, String> {
        let journal_path = if let Some(p) = journal_path_opt {
            PathBuf::from(p)
        } else if let Some(root) = root_path_opt {
            Path::new(root).join(JOURNAL_FILENAME)
        } else {
            PathBuf::from(JOURNAL_FILENAME)
        };

        if !journal_path.exists() {
            return Err(format!("Journal file not found at `{}`", journal_path.display()));
        }

        let journal_str = fs::read_to_string(&journal_path)
            .map_err(|e| format!("Failed to read journal: {}", e))?;

        let mut journal: JournalEntry = serde_json::from_str(&journal_str)
            .map_err(|e| format!("Failed to parse journal JSON: {}", e))?;

        if journal.status == JournalStatus::RolledBack {
            return Err(format!(
                "Transaction `{}` has already been rolled back.",
                journal.transaction_id
            ));
        }

        let mut restored_count = 0;

        // 1. Delete all files that were created or moved to new locations
        for created_file in &journal.created_files {
            let p = Path::new(created_file);
            if p.exists() {
                let _ = fs::remove_file(p);
            }
        }

        // 2. Restore all original files from backup
        for (original_path_str, original_content) in &journal.original_files_backup {
            let p = Path::new(original_path_str);
            if let Some(parent) = p.parent() {
                let _ = fs::create_dir_all(parent);
            }
            fs::write(p, original_content)
                .map_err(|e| format!("Failed to restore file `{}`: {}", original_path_str, e))?;
            restored_count += 1;
        }

        // 3. Mark journal as rolled back
        journal.status = JournalStatus::RolledBack;
        let updated_json = serde_json::to_string_pretty(&journal)
            .map_err(|e| format!("Failed to serialize updated journal: {}", e))?;
        let _ = fs::write(&journal_path, updated_json);

        Ok(RollbackResult {
            success: true,
            restored_files_count: restored_count,
            transaction_id: journal.transaction_id.clone(),
            message: format!(
                "Successfully rolled back transaction `{}`. Restored {} files.",
                journal.transaction_id, restored_count
            ),
        })
    }
}
