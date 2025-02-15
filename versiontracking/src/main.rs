use cargo_metadata::{MetadataCommand, Package};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Select};
use owo_colors::OwoColorize;
use reqwest::Client;
use semver::Version;
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;
use walkdir::WalkDir;

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

    #[error("No Cargo.toml files found in the specified directory")]
    NoCargoFiles,

    #[error("User cancelled selection")]
    SelectionCancelled,

    #[error("Rate limit exceeded for crates.io API")]
    RateLimitExceeded,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    manifest_path: Option<String>,

    #[arg(short, long)]
    json_output: bool,

    #[arg(short, long, default_value = "..")]
    search_path: String,
}

#[derive(Serialize)]
struct DependencyInfo {
    name: String,
    current_version: String,
    latest_version: Option<String>,
    update_recommended: bool,
    vulnerabilities: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    dependencies: Vec<DependencyInfo>,
    audit_output: String,
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

fn find_cargo_files(search_path: &str) -> Result<Vec<PathBuf>, VersionError> {
    let mut cargo_files = Vec::new();

    for entry in WalkDir::new(search_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "Cargo.toml" {
            cargo_files.push(entry.path().to_path_buf());
        }
    }

    if cargo_files.is_empty() {
        return Err(VersionError::NoCargoFiles);
    }

    Ok(cargo_files)
}

fn select_cargo_file(cargo_files: Vec<PathBuf>) -> Result<PathBuf, VersionError> {
    let options: Vec<String> = cargo_files
        .iter()
        .map(|path| path.display().to_string())
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a Cargo.toml file to analyze")
        .items(&options)
        .default(0)
        .interact()
        .map_err(|_| VersionError::SelectionCancelled)?;

    Ok(cargo_files[selection].clone())
}

#[tokio::main]
async fn main() -> Result<(), VersionError> {
    let args = Args::parse();

    // Get manifest path either from argument or by searching
    let manifest_path = match args.manifest_path {
        Some(path) => PathBuf::from(path),
        None => {
            let cargo_files = find_cargo_files(&args.search_path)?;
            select_cargo_file(cargo_files)?
        }
    };

    println!("Analyzing {}", manifest_path.display().bold());

    // Get metadata
    let metadata = get_metadata(manifest_path.to_str().unwrap())?;
    let mut report = Report {
        dependencies: Vec::new(),
        audit_output: String::new(),
    };

    let client = Client::builder()
        .user_agent("cargo-versioncheck/0.1.0")
        .build()?;

    // Process packages concurrently
    let mut tasks = Vec::new();
    for package in metadata.packages {
        if let Some(source) = package.source.as_ref() {
            if source.is_crates_io() {
                let client = client.clone();
                tasks.push(tokio::spawn(async move {
                    get_dependency_info(&package, &client).await
                }));
            }
        }
    }

    // Collect results
    for task in tasks {
        if let Ok(Some(dep)) = task.await {
            report.dependencies.push(dep);
        }
    }

    report.dependencies.sort_by(|a, b| a.name.cmp(&b.name));

    // Run cargo audit
    report.audit_output = run_cargo_audit()?;

    // Output report
    output_report(&report, args.json_output)?;

    Ok(())
}

fn get_metadata(manifest_path: &str) -> Result<cargo_metadata::Metadata, VersionError> {
    Ok(MetadataCommand::new().manifest_path(manifest_path).exec()?)
}

async fn get_dependency_info(package: &Package, client: &Client) -> Option<DependencyInfo> {
    let name = &package.name;
    let current_version = package.version.to_string();

    let url = format!("https://crates.io/api/v1/crates/{}", name);
    let response = client.get(&url).send().await.ok()?;

    if response.status().is_client_error() {
        if response.status().as_u16() == 429 {
            eprintln!(
                "{}",
                "Rate limit exceeded for crates.io API. Some version information may be missing."
                    .red()
            );
        }
        return Some(DependencyInfo {
            name: name.clone(),
            current_version,
            latest_version: None,
            update_recommended: false,
            vulnerabilities: Vec::new(),
        });
    }

    if let Ok(json) = response.json::<serde_json::Value>().await {
        if let Some(crate_info) = json.get("crate") {
            return process_crate_info(name, &current_version, crate_info);
        }
    }

    Some(DependencyInfo {
        name: name.clone(),
        current_version,
        latest_version: None,
        update_recommended: false,
        vulnerabilities: Vec::new(),
    })
}

fn process_crate_info(
    name: &str,
    current_version: &str,
    crate_info: &serde_json::Value,
) -> Option<DependencyInfo> {
    let current_version = Version::parse(current_version).ok()?;

    let latest_version = crate_info
        .get("max_version")
        .and_then(|v| v.as_str())
        .and_then(|v| Version::parse(v).ok());

    let update_recommended = latest_version
        .as_ref()
        .map(|v| v > &current_version)
        .unwrap_or(false);

    Some(DependencyInfo {
        name: name.to_string(),
        current_version: current_version.to_string(),
        latest_version: latest_version.map(|v| v.to_string()),
        update_recommended,
        vulnerabilities: Vec::new(),
    })
}

fn run_cargo_audit() -> Result<String, VersionError> {
    let output = Command::new("cargo").arg("audit").output()?;
    let mut audit_output = String::from_utf8_lossy(&output.stdout).to_string();

    if !output.status.success() {
        audit_output.push_str(&String::from_utf8_lossy(&output.stderr));
    }

    Ok(audit_output)
}

fn output_report(report: &Report, json_output: bool) -> Result<(), VersionError> {
    if json_output {
        println!("{}", serde_json::to_string_pretty(report)?);
    } else {
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

        // Print summary
        println!("\n{}", "Summary:".bold());
        println!(
            "Total dependencies: {}",
            report.dependencies.len().to_string().bold()
        );
        println!(
            "Updates available: {}",
            updates.len().to_string().yellow().bold()
        );
        println!("Up to date: {}", current.len().to_string().green().bold());

        if !report.audit_output.is_empty() {
            println!("\n{}", "Security Audit Results:".red().bold());
            println!("{}", "=====================".red());
            println!("{}", report.audit_output);
        }
    }
    Ok(())
}

fn status_text(text: &str, update_needed: bool) -> String {
    if update_needed {
        text.yellow().bold().to_string()
    } else {
        text.green().bold().to_string()
    }
}
