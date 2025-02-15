use clap::{Parser, Subcommand};
use owo_colors::OwoColorize;
use serde_json;
use std::path::PathBuf;
use versiontracking::{VersionError, VersionTracker};

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

#[tokio::main]
async fn main() -> Result<(), VersionError> {
    let args = Args::parse();
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("versiontracking");

    let tracker = VersionTracker::new(Some(cache_dir))?;

    match args.command {
        Commands::Check {
            manifest_path,
            json_output,
            recursive,
            search_path,
        } => {
            let reports = tracker
                .check_versions(manifest_path, json_output, recursive, search_path)
                .await?;
            if json_output {
                println!("{}", serde_json::to_string_pretty(&reports)?);
            } else {
                for (path, report) in reports {
                    println!("\n{}", "=".repeat(50));
                    println!("Results for: {}", path.display().bold());
                    println!("{}", "=".repeat(50));

                    let (updates, current) = report.partition_dependencies();
                    if !updates.is_empty() {
                        println!("\n{}", "Updates Available:".yellow().bold());
                        println!("{}", "=================".yellow());
                        for dep in &updates {
                            println!(
                                "{}: {} → {}",
                                dep.name.blue().bold(),
                                dep.current_version,
                                dep.latest_version
                                    .as_deref()
                                    .unwrap_or("unknown")
                                    .yellow()
                                    .bold()
                            );
                        }
                    }

                    if !current.is_empty() {
                        println!("\n{}", "Up to Date:".green().bold());
                        println!("{}", "==========".green());
                        for dep in &current {
                            println!("{}: {}", dep.name.blue().bold(), dep.current_version);
                        }
                    }
                }
            }
        }
        Commands::Compare { old_file, new_file } => {
            let changes = tracker.compare_files(&old_file, &new_file)?;
            if changes.is_empty() {
                println!("{}", "No breaking changes detected.".green().bold());
            } else {
                println!("\n{}", "Breaking Changes Detected:".red().bold());
                println!("{}", "=======================".red());

                for change in changes {
                    println!("{:?}", change);
                }
            }
        }
        Commands::Analyze {
            manifest_path,
            recursive,
        } => {
            tracker
                .analyze_dependencies(manifest_path, recursive)
                .await?;
        }
        Commands::Search {
            query,
            categories,
            json_output,
        } => {
            let results = tracker
                .search_crates(
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

            let metadata =
                versiontracking::version_management::get_metadata(manifest_path.to_str().unwrap())?;
            let deps: Vec<String> = metadata
                .packages
                .iter()
                .flat_map(|p| p.dependencies.iter().map(|d| d.name.clone()))
                .collect();

            let suggestions = tracker.get_recommendations(&deps).await?;

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
            let (metadata, extract_path) = tracker.inspect_crate(&name, version).await?;

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
