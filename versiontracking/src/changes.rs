use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChangeError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Failed to create changes directory")]
    DirectoryCreationError,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeFix {
    pub file_path: String,
    pub line_number: usize,
    pub old_code: String,
    pub suggested_fix: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DependencyChange {
    pub name: String,
    pub old_version: String,
    pub new_version: String,
    pub breaking_changes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChangeRecord {
    pub timestamp: DateTime<Utc>,
    pub dependency_changes: Vec<DependencyChange>,
    pub affected_files: Vec<String>,
    pub required_fixes: Vec<CodeFix>,
    pub commit_hash: Option<String>, // If the change is associated with a git commit
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeLog {
    pub project_name: String,
    pub records: Vec<ChangeRecord>,
}

pub struct ChangeTracker {
    changes_dir: PathBuf,
    current_log: ChangeLog,
}

impl ChangeTracker {
    pub fn new(project_root: &Path) -> Result<Self, ChangeError> {
        let changes_dir = project_root.join(".changes");
        fs::create_dir_all(&changes_dir).map_err(|_| ChangeError::DirectoryCreationError)?;

        let log_path = changes_dir.join("changelog.json");
        let current_log = if log_path.exists() {
            let content = fs::read_to_string(&log_path)?;
            serde_json::from_str(&content)?
        } else {
            ChangeLog {
                project_name: project_root
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                records: Vec::new(),
            }
        };

        Ok(Self {
            changes_dir,
            current_log,
        })
    }

    pub fn add_record(&mut self, record: ChangeRecord) -> Result<(), ChangeError> {
        self.current_log.records.push(record);
        self.save()?;
        Ok(())
    }

    fn save(&self) -> Result<(), ChangeError> {
        let log_path = self.changes_dir.join("changelog.json");
        let content = serde_json::to_string_pretty(&self.current_log)?;
        fs::write(log_path, content)?;
        Ok(())
    }

    /// Archives the current change record and creates a detailed report
    #[allow(dead_code)] // Will be used in future expansions
    pub fn archive_changes(&self, record: &ChangeRecord) -> Result<PathBuf, ChangeError> {
        let timestamp = record.timestamp.format("%Y%m%d_%H%M%S").to_string();
        let archive_dir = self.changes_dir.join("archives").join(&timestamp);
        fs::create_dir_all(&archive_dir)?;

        // Save detailed change information
        let details_path = archive_dir.join("details.json");
        fs::write(&details_path, serde_json::to_string_pretty(&record)?)?;

        // Generate a human-readable report
        let report = self.generate_report(record);
        fs::write(archive_dir.join("report.md"), report)?;

        // Create fix suggestions file if there are any
        if !record.required_fixes.is_empty() {
            let fixes = self.generate_fix_guide(record);
            fs::write(archive_dir.join("fixes.md"), fixes)?;
        }

        Ok(archive_dir)
    }

    #[allow(dead_code)] // Will be used in future expansions
    fn generate_report(&self, record: &ChangeRecord) -> String {
        let mut report = String::new();

        report.push_str(&format!(
            "# Change Report - {}\n\n",
            record.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Dependency Changes
        report.push_str("## Dependency Changes\n\n");
        for dep in &record.dependency_changes {
            report.push_str(&format!("### {}\n", dep.name));
            report.push_str(&format!(
                "- Updated: {} → {}\n",
                dep.old_version, dep.new_version
            ));
            if !dep.breaking_changes.is_empty() {
                report.push_str("- Breaking Changes:\n");
                for change in &dep.breaking_changes {
                    report.push_str(&format!("  - {}\n", change));
                }
            }
            report.push_str("\n");
        }

        // Affected Files
        report.push_str("## Affected Files\n\n");
        for file in &record.affected_files {
            report.push_str(&format!("- {}\n", file));
        }
        report.push_str("\n");

        // Required Fixes Summary
        if !record.required_fixes.is_empty() {
            report.push_str("## Required Fixes Summary\n\n");
            for fix in &record.required_fixes {
                report.push_str(&format!("### {}\n", fix.file_path));
                report.push_str(&format!("- Line: {}\n", fix.line_number));
                report.push_str(&format!("- Reason: {}\n\n", fix.reason));
            }
        }

        report
    }

    #[allow(dead_code)] // Will be used in future expansions
    fn generate_fix_guide(&self, record: &ChangeRecord) -> String {
        let mut guide = String::new();

        guide.push_str("# Code Fix Guide\n\n");
        guide
            .push_str("This guide provides detailed instructions for fixing breaking changes.\n\n");

        // Group fixes by file
        let mut fixes_by_file: HashMap<&str, Vec<&CodeFix>> = HashMap::new();
        for fix in &record.required_fixes {
            fixes_by_file.entry(&fix.file_path).or_default().push(fix);
        }

        // Generate fixes for each file
        for (file, fixes) in fixes_by_file {
            guide.push_str(&format!("## {}\n\n", file));

            for fix in fixes {
                guide.push_str(&format!("### Line {}\n\n", fix.line_number));
                guide.push_str("#### Current Code:\n");
                guide.push_str("```rust\n");
                guide.push_str(&fix.old_code);
                guide.push_str("\n```\n\n");

                guide.push_str("#### Suggested Fix:\n");
                guide.push_str("```rust\n");
                guide.push_str(&fix.suggested_fix);
                guide.push_str("\n```\n\n");

                guide.push_str(&format!("#### Reason:\n{}\n\n", fix.reason));
            }
        }

        guide.push_str("\n## How to Apply Fixes\n\n");
        guide.push_str("1. Review each suggested fix carefully\n");
        guide.push_str("2. Test changes in isolation when possible\n");
        guide.push_str("3. Update tests to reflect new requirements\n");
        guide.push_str("4. Run the full test suite after applying fixes\n");

        guide
    }
}
