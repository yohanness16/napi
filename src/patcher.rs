use crate::types::{DiffHunk, FileDiff, PatchAction};
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use similar::{ChangeTag, TextDiff};
use std::path::Path;

pub struct BytePatcher;

impl BytePatcher {
    /// Apply byte-level AST patches to file content preserving whitespace, formatting, and quotes
    pub fn patch_content(original_content: &str, patches: &[PatchAction]) -> Result<String, String> {
        if patches.is_empty() {
            return Ok(original_content.to_string());
        }

        // Sort patches in descending order of span_start to prevent offset drift
        let mut sorted_patches = patches.to_vec();
        sorted_patches.sort_by(|a, b| b.span_start.cmp(&a.span_start));

        let mut bytes = original_content.as_bytes().to_vec();

        for patch in &sorted_patches {
            let start = patch.span_start as usize;
            let end = patch.span_end as usize;

            if start > bytes.len() || end > bytes.len() || start > end {
                return Err(format!(
                    "Invalid byte span [{}..{}] for content length {}",
                    start,
                    end,
                    bytes.len()
                ));
            }

            // Detect quote style in the original slice
            let original_slice = match std::str::from_utf8(&bytes[start..end]) {
                Ok(s) => s,
                Err(_) => return Err("Invalid UTF-8 in span target".to_string()),
            };

            let quote_char = if original_slice.starts_with('\'') && original_slice.ends_with('\'') {
                "'"
            } else if original_slice.starts_with('\"') && original_slice.ends_with('\"') {
                "\""
            } else if original_slice.starts_with('`') && original_slice.ends_with('`') {
                "`"
            } else {
                "\""
            };

            // Format replacement string with preserved quotes
            let replacement_string = format!("{}{}{}", quote_char, patch.replacement_specifier, quote_char);
            let replacement_bytes = replacement_string.as_bytes();

            // Splice bytes
            bytes.splice(start..end, replacement_bytes.iter().cloned());
        }

        let patched_content = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => return Err(format!("UTF-8 conversion error after patching: {}", e)),
        };

        Ok(patched_content)
    }

    /// Verify that the patched content parses cleanly with Oxc
    pub fn verify_syntax(content: &str, file_path: &Path) -> Result<(), String> {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(file_path).unwrap_or_default();
        let ret = Parser::new(&allocator, content, source_type).parse();

        if !ret.errors.is_empty() {
            let error_msgs: Vec<String> = ret.errors.iter().map(|e| e.to_string()).collect();
            return Err(format!(
                "Syntax validation failed on patched file `{}`: {}",
                file_path.display(),
                error_msgs.join("; ")
            ));
        }

        Ok(())
    }

    /// Generate unified diff preview for a file
    pub fn generate_unified_diff(
        old_content: &str,
        new_content: &str,
        file_path: &str,
        relative_path: &str,
        is_new: bool,
        is_deleted: bool,
        is_moved: bool,
        old_path: Option<String>,
        new_path: Option<String>,
    ) -> FileDiff {
        let diff = TextDiff::from_lines(old_content, new_content);
        let mut unified_diff = String::new();
        let mut additions = 0;
        let mut deletions = 0;
        let mut hunks = Vec::new();

        let old_header = old_path.as_deref().unwrap_or(file_path);
        let new_header = new_path.as_deref().unwrap_or(file_path);

        unified_diff.push_str(&format!("--- a/{}\n", old_header));
        unified_diff.push_str(&format!("+++ b/{}\n", new_header));

        for hunk in diff.unified_diff().context_radius(3).iter_hunks() {
            let mut hunk_lines = Vec::new();
            let header = format!("{}", hunk.header());
            unified_diff.push_str(&header);
            unified_diff.push('\n');

            let mut old_count = 0;
            let mut new_count = 0;
            let mut first_old = None;
            let mut first_new = None;

            for change in hunk.iter_changes() {
                if let Some(idx) = change.old_index() {
                    if first_old.is_none() {
                        first_old = Some(idx);
                    }
                    old_count += 1;
                }
                if let Some(idx) = change.new_index() {
                    if first_new.is_none() {
                        first_new = Some(idx);
                    }
                    new_count += 1;
                }

                let sign = match change.tag() {
                    ChangeTag::Delete => {
                        deletions += 1;
                        "-"
                    }
                    ChangeTag::Insert => {
                        additions += 1;
                        "+"
                    }
                    ChangeTag::Equal => " ",
                };
                let line = format!("{}{}", sign, change.value());
                unified_diff.push_str(&line);
                hunk_lines.push(line);
            }

            hunks.push(DiffHunk {
                old_start: first_old.unwrap_or(0) as u32,
                old_lines: old_count as u32,
                new_start: first_new.unwrap_or(0) as u32,
                new_lines: new_count as u32,
                header,
                lines: hunk_lines,
            });
        }

        FileDiff {
            file_path: file_path.to_string(),
            relative_path: relative_path.to_string(),
            is_new_file: is_new,
            is_deleted_file: is_deleted,
            is_moved,
            old_path,
            new_path,
            unified_diff,
            additions,
            deletions,
            hunks,
        }
    }
}
