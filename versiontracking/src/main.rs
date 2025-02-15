use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use serde::Serialize;
use std::path::PathBuf;
use thiserror::Error;

mod analysis;
mod ast_comparator;
mod ast_extractor;
mod changes;
mod crates_io;
mod version_management;

#[derive(Error, Debug)]
pub enum VersionError {
    #[error("Failed to execute cargo metadata: {0}")]
    MetadataError(#[from] cargo_metadata::Error),

    #[error("Failed to execute cargo audit: {0}")]
    AuditError(#[from] std::io::Error),

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

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check for version updates and security vulnerabilities
    Check {
        #[arg(short, long)]
        manifest_path: Option<String>,

        #[arg(short, long)]
        json_output: bool,

        #[arg(short, long)]
        recursive: bool,

        #[arg(short, long, default_value = "..")]
        search_path: String,
    },
    /// Compare two Rust source files for breaking changes
    Compare {
        /// Path to the old version of the source file
        old_file: String,
        /// Path to the new version of the source file
        new_file: String,
    },
    /// Analyze breaking changes from dependency updates
    Analyze {
        #[arg(short, long)]
        manifest_path: Option<String>,

        #[arg(short, long)]
        recursive: bool,
    },
    /// Search for crates on crates.io
    Search {
        /// Search query
        query: String,

        #[arg(short, long)]
        categories: Vec<String>,

        #[arg(short, long)]
        json_output: bool,
    },
    /// Get crate recommendations based on current dependencies
    Recommend {
        #[arg(short, long)]
        manifest_path: Option<String>,

        #[arg(short, long)]
        json_output: bool,
    },
    /// Download and analyze a specific crate
    Inspect {
        /// Name of the crate
        name: String,
        /// Version of the crate (optional)
        version: Option<String>,
    },
}

#[derive(Serialize)]
pub struct DependencyInfo {
    pub name: String,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_recommended: bool,
    pub vulnerabilities: Vec<String>,
}

#[derive(Serialize)]
pub struct Report {
    pub dependencies: Vec<DependencyInfo>,
    pub audit_output: String,
}

impl Report {
    fn partition_dependencies(&self) -> (Vec<&DependencyInfo>, Vec<&DependencyInfo>) {
        let (updates, current): (Vec<_>, Vec<_>) = self
            .dependencies
            .iter()
            .partition(|dep| dep.update_recommended);
        (updates, current)
    }
}

#[tokio::main]
async fn main() -> Result<(), VersionError> {
    let args = Args::parse();
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("versiontracking");
    let crates_client = crates_io::CratesIoClient::new(cache_dir)?;

    match args.command {
        Commands::Check {
            manifest_path,
            json_output,
            recursive,
            search_path,
        } => {
            version_management::check_versions(manifest_path, json_output, recursive, search_path)
                .await?;
        }
        Commands::Compare { old_file, new_file } => {
            version_management::compare_files(&old_file, &new_file)?;
        }
        Commands::Analyze {
            manifest_path,
            recursive,
        } => {
            version_management::analyze_dependencies(manifest_path, recursive).await?;
        }
        Commands::Search {
            query,
            categories,
            json_output,
        } => {
            let results = crates_client
                .search(
                    &query,
                    &categories.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                )
                .await?;

            if json_output {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                println!("\n{}", "Search Results:".blue().bold());
                println!("{}", "==============".blue());
                println!("Found {} crates\n", results.total);

                for krate in &results.crates {
                    println!("{}", krate.name.green().bold());
                    if let Some(desc) = &krate.description {
                        println!("{}", desc);
                    }
                    println!("Downloads: {}", krate.downloads);
                    if !krate.categories.is_empty() {
                        println!("Categories: {}", krate.categories.join(", "));
                    }
                    println!();
                }
            }
        }
        Commands::Recommend {
            manifest_path,
            json_output,
        } => {
            let manifest_path = manifest_path
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("Cargo.toml"));

            let metadata = version_management::get_metadata(manifest_path.to_str().unwrap())?;
            let deps: Vec<String> = metadata
                .packages
                .iter()
                .flat_map(|p| p.dependencies.iter().map(|d| d.name.clone()))
                .collect();

            let suggestions = crates_client.suggest_dependencies(&deps).await?;

            if json_output {
                println!("{}", serde_json::to_string_pretty(&suggestions)?);
            } else {
                println!("\n{}", "Recommendations:".blue().bold());
                println!("{}", "===============".blue());

                for (dep, similar) in &suggestions {
                    println!("\nBased on {}: ", dep.green().bold());
                    for krate in similar.iter().take(5) {
                        println!("  - {} ({} downloads)", krate.name, krate.downloads);
                        if let Some(desc) = &krate.description {
                            println!("    {}", desc);
                        }
                    }
                }
            }
        }
        Commands::Inspect { name, version } => {
            let metadata = crates_client.get_crate_metadata(&name).await?;
            let version = version.unwrap_or_else(|| metadata.versions[0].clone());

            println!("Downloading {} v{}", name.green().bold(), version);
            let crate_path = crates_client.download_crate(&name, &version).await?;
            let extract_path = crates_io::extract_crate(&crate_path)?;

            println!("\n{}", "Crate Info:".blue().bold());
            println!("{}", "==========".blue());
            println!("Name: {}", metadata.name);
            if let Some(desc) = metadata.description {
                println!("Description: {}", desc);
            }
            println!("Downloads: {}", metadata.downloads);
            if let Some(docs) = metadata.documentation {
                println!("Documentation: {}", docs);
            }
            if let Some(repo) = metadata.repository {
                println!("Repository: {}", repo);
            }
            println!("\nExtracted to: {}", extract_path.display());
        }
    }

    Ok(())
}
