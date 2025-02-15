pub mod analysis;
pub mod ast_comparator;
pub mod ast_extractor;
pub mod changes;
pub mod crates_io;
pub mod version_management;

use serde::Serialize;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VersionError {
    #[error("Failed to execute cargo metadata: {0}")]
    MetadataError(#[from] cargo_metadata::Error),

    #[error("Failed to execute cargo audit: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to parse version: {0}")]
    VersionParseError(#[from] semver::Error),

    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("AST error: {0}")]
    AstError(#[from] ast_extractor::AstError),

    #[error("No Cargo.toml files found in the specified directory")]
    NoCargoFiles,

    #[error("User cancelled selection")]
    SelectionCancelled,

    #[error("Rate limit exceeded for crates.io API")]
    RateLimitExceeded,

    #[error("Failed to analyze dependencies: {0}")]
    AnalysisError(String),

    #[error("Crate operation failed: {0}")]
    CrateError(#[from] crates_io::CrateError),
}

#[derive(Serialize, Debug, Clone)]
pub struct DependencyInfo {
    pub name: String,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_recommended: bool,
    pub vulnerabilities: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Report {
    pub dependencies: Vec<DependencyInfo>,
    pub audit_output: String,
}

impl Report {
    pub fn partition_dependencies(&self) -> (Vec<&DependencyInfo>, Vec<&DependencyInfo>) {
        let (updates, current): (Vec<_>, Vec<_>) = self
            .dependencies
            .iter()
            .partition(|dep| dep.update_recommended);
        (updates, current)
    }

    pub fn new() -> Self {
        Self {
            dependencies: Vec::new(),
            audit_output: String::new(),
        }
    }
}

/// The main interface for version tracking functionality
pub struct VersionTracker {
    crates_client: crates_io::CratesIoClient,
}

impl VersionTracker {
    /// Create a new VersionTracker instance
    pub fn new(cache_dir: Option<PathBuf>) -> Result<Self, VersionError> {
        let cache_dir = cache_dir.unwrap_or_else(|| {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from(".cache"))
                .join("versiontracking")
        });

        Ok(Self {
            crates_client: crates_io::CratesIoClient::new(cache_dir)?,
        })
    }

    /// Check for updates in the specified manifest
    pub async fn check_versions(
        &self,
        manifest_path: Option<String>,
        json_output: bool,
        recursive: bool,
        search_path: String,
    ) -> Result<Vec<(PathBuf, Report)>, VersionError> {
        version_management::check_versions(manifest_path, json_output, recursive, search_path).await
    }

    /// Compare two Rust source files for breaking changes
    pub fn compare_files(
        &self,
        old_file: &str,
        new_file: &str,
    ) -> Result<Vec<ast_comparator::BreakingChange>, VersionError> {
        let (old_ast, new_ast) = ast_extractor::extract_asts_for_comparison(old_file, new_file)?;
        let tracker = ast_comparator::compare_asts(&old_ast, &new_ast);
        Ok(tracker.changes)
    }

    /// Analyze dependencies for breaking changes
    pub async fn analyze_dependencies(
        &self,
        manifest_path: Option<String>,
        recursive: bool,
    ) -> Result<(), VersionError> {
        version_management::analyze_dependencies(manifest_path, recursive).await
    }

    /// Search for crates on crates.io
    pub async fn search_crates(
        &self,
        query: &str,
        categories: &[&str],
    ) -> Result<crates_io::SearchResults, VersionError> {
        Ok(self.crates_client.search(query, categories).await?)
    }

    /// Get recommendations for related crates
    pub async fn get_recommendations(
        &self,
        dependencies: &[String],
    ) -> Result<std::collections::HashMap<String, Vec<crates_io::CrateMetadata>>, VersionError>
    {
        Ok(self
            .crates_client
            .suggest_dependencies(dependencies)
            .await?)
    }

    /// Inspect a specific crate
    pub async fn inspect_crate(
        &self,
        name: &str,
        version: Option<String>,
    ) -> Result<(crates_io::CrateMetadata, PathBuf), VersionError> {
        let metadata = self.crates_client.get_crate_metadata(name).await?;
        let version = version.unwrap_or_else(|| metadata.versions[0].clone());
        let crate_path = self.crates_client.download_crate(name, &version).await?;
        let extract_path = crates_io::extract_crate(&crate_path)?;

        Ok((metadata, extract_path))
    }
}
