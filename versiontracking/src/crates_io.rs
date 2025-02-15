use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct CrateMetadata {
    pub name: String,
    pub description: Option<String>,
    pub downloads: u64,
    pub recent_downloads: Option<u64>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    #[serde(default)]
    pub versions: Vec<String>,
    pub documentation: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    crates: Vec<CrateMetadata>,
    meta: ResponseMeta,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResponseMeta {
    total: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResults {
    pub total: u64,
    pub crates: Vec<CrateMetadata>,
}

#[derive(Debug, thiserror::Error)]
pub enum CrateError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to create directory: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Failed to download crate: {0}")]
    DownloadError(String),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub struct CratesIoClient {
    client: Client,
    cache_dir: PathBuf,
}

impl CratesIoClient {
    pub fn new(cache_dir: PathBuf) -> Result<Self, CrateError> {
        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            client: Client::builder()
                .user_agent("versiontracking/0.1.0")
                .build()?,
            cache_dir,
        })
    }

    /// Search for crates based on keywords and categories
    pub async fn search(
        &self,
        query: &str,
        categories: &[&str],
    ) -> Result<SearchResults, CrateError> {
        let mut url = format!("https://crates.io/api/v1/crates?q={}&per_page=10", query);

        if !categories.is_empty() {
            url.push_str("&categories=");
            url.push_str(&categories.join(","));
        }

        let response = self.client.get(&url).send().await?;

        if response.status().as_u16() == 429 {
            return Err(CrateError::RateLimitExceeded);
        }

        let api_response: ApiResponse = response.json().await?;

        Ok(SearchResults {
            total: api_response.meta.total,
            crates: api_response.crates,
        })
    }

    /// Download a specific version of a crate
    pub async fn download_crate(&self, name: &str, version: &str) -> Result<PathBuf, CrateError> {
        let cache_path = self.cache_dir.join(format!("{}-{}.crate", name, version));

        if cache_path.exists() {
            return Ok(cache_path);
        }

        let url = format!(
            "https://crates.io/api/v1/crates/{}/{}/download",
            name, version
        );

        let response = self.client.get(&url).send().await?;

        if response.status().as_u16() == 429 {
            return Err(CrateError::RateLimitExceeded);
        }

        let mut file = tokio::fs::File::create(&cache_path).await?;
        let mut content = response.bytes().await?;
        file.write_all_buf(&mut content).await?;

        Ok(cache_path)
    }

    /// Get detailed metadata for a specific crate
    pub async fn get_crate_metadata(&self, name: &str) -> Result<CrateMetadata, CrateError> {
        let url = format!("https://crates.io/api/v1/crates/{}", name);
        let response = self.client.get(&url).send().await?;

        if response.status().as_u16() == 429 {
            return Err(CrateError::RateLimitExceeded);
        }

        let json: serde_json::Value = response.json().await?;
        let crate_data = json
            .get("crate")
            .ok_or_else(|| CrateError::DownloadError("Invalid response format".to_string()))?;

        serde_json::from_value(crate_data.clone()).map_err(CrateError::JsonError)
    }

    /// Find similar crates based on categories and keywords
    pub async fn find_similar_crates(
        &self,
        crate_name: &str,
    ) -> Result<Vec<CrateMetadata>, CrateError> {
        let metadata = self.get_crate_metadata(crate_name).await?;
        let mut similar_crates = Vec::new();

        // Search by categories
        if !metadata.categories.is_empty() {
            let results = self
                .search(
                    "",
                    &metadata
                        .categories
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>(),
                )
                .await?;
            similar_crates.extend(results.crates);
        }

        // Search by keywords
        if !metadata.keywords.is_empty() {
            for keyword in &metadata.keywords {
                let results = self.search(keyword, &[]).await?;
                similar_crates.extend(results.crates);
            }
        }

        // Remove duplicates and the original crate
        similar_crates.sort_by(|a, b| b.downloads.cmp(&a.downloads));
        similar_crates.dedup_by(|a, b| a.name == b.name);
        similar_crates.retain(|c| c.name != crate_name);

        Ok(similar_crates)
    }

    /// Analyze project dependencies and suggest related crates
    pub async fn suggest_dependencies(
        &self,
        dependencies: &[String],
    ) -> Result<HashMap<String, Vec<CrateMetadata>>, CrateError> {
        let mut suggestions = HashMap::new();

        for dep in dependencies {
            let similar = self.find_similar_crates(dep).await?;
            suggestions.insert(dep.clone(), similar);
        }

        Ok(suggestions)
    }
}

/// Helper function to extract downloaded crate
pub fn extract_crate(crate_path: &Path) -> Result<PathBuf, CrateError> {
    let extract_dir = crate_path.parent().unwrap().join(
        crate_path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );

    if !extract_dir.exists() {
        fs::create_dir_all(&extract_dir)?;
        let file = fs::File::open(crate_path)?;
        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(file));
        archive.unpack(&extract_dir)?;
    }

    Ok(extract_dir)
}
