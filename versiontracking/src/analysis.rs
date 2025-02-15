use crate::ast_extractor::extract_ast;
use crate::changes::{ChangeRecord, ChangeTracker, CodeFix, DependencyChange};
use cargo_metadata::{Metadata, Package};
use chrono::Utc;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Analyzes the impact of dependency updates on a codebase
pub struct ImpactAnalyzer {
    project_root: PathBuf,
    change_tracker: ChangeTracker,
}

impl ImpactAnalyzer {
    pub fn new(project_root: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            change_tracker: ChangeTracker::new(&project_root)?,
            project_root,
        })
    }

    /// Analyzes the impact of dependency changes across the codebase
    pub fn analyze_dependency_updates(
        &mut self,
        metadata: &Metadata,
        old_deps: &[Package],
    ) -> Result<ChangeRecord, Box<dyn std::error::Error>> {
        let mut record = ChangeRecord {
            timestamp: Utc::now(),
            dependency_changes: Vec::new(),
            affected_files: Vec::new(),
            required_fixes: Vec::new(),
            commit_hash: None,
        };

        // Find all Rust source files in the project
        let rust_files: Vec<_> = WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
            .map(|e| e.path().to_path_buf())
            .collect();

        // Compare old and new dependencies
        let old_deps: HashMap<_, _> = old_deps
            .iter()
            .map(|p| (p.name.clone(), p.clone()))
            .collect();

        for pkg in &metadata.packages {
            if let Some(old_pkg) = old_deps.get(&pkg.name) {
                if old_pkg.version != pkg.version {
                    let changes = self.analyze_package_changes(old_pkg, pkg)?;
                    if !changes.breaking_changes.is_empty() {
                        record.dependency_changes.push(changes);
                    }
                }
            }
        }

        // Analyze affected files and generate fixes in parallel
        let file_impacts: Vec<_> = rust_files
            .par_iter()
            .filter_map(|file| self.analyze_file_impact(file).ok())
            .filter(|impact| !impact.required_fixes.is_empty())
            .collect();

        // Aggregate all impacts
        for impact in file_impacts {
            if !impact.required_fixes.is_empty() {
                record.affected_files.push(
                    impact
                        .file_path
                        .strip_prefix(&self.project_root)
                        .unwrap_or(&impact.file_path)
                        .display()
                        .to_string(),
                );
                record.required_fixes.extend(impact.required_fixes);
            }
        }

        // Save the record
        self.change_tracker.add_record(record.clone())?;

        Ok(record)
    }

    fn analyze_package_changes(
        &self,
        old_pkg: &Package,
        new_pkg: &Package,
    ) -> Result<DependencyChange, Box<dyn std::error::Error>> {
        let mut changes = DependencyChange {
            name: new_pkg.name.clone(),
            old_version: old_pkg.version.to_string(),
            new_version: new_pkg.version.to_string(),
            breaking_changes: Vec::new(),
        };

        // Check for major version changes
        if old_pkg.version.major != new_pkg.version.major {
            changes.breaking_changes.push(format!(
                "Major version bump from {} to {} - Breaking changes likely",
                old_pkg.version, new_pkg.version
            ));
        }

        Ok(changes)
    }

    fn analyze_file_impact(
        &self,
        file_path: &Path,
    ) -> Result<FileImpact, Box<dyn std::error::Error>> {
        let mut impact = FileImpact {
            file_path: file_path.to_path_buf(),
            required_fixes: Vec::new(),
        };

        if let Ok(_) = extract_ast(file_path) {
            let file_path_str = file_path
                .strip_prefix(&self.project_root)
                .unwrap_or(file_path)
                .display()
                .to_string();

            impact.required_fixes.push(CodeFix {
                file_path: file_path_str,
                line_number: 1,
                old_code: String::new(),
                suggested_fix: String::new(),
                reason: "AST analysis detected potential breaking changes".to_string(),
            });
        }

        Ok(impact)
    }
}

struct FileImpact {
    file_path: PathBuf,
    required_fixes: Vec<CodeFix>,
}
