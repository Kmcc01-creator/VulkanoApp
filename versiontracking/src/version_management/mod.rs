use crate::VersionError;
use crate::{DependencyInfo, Report};
use cargo_metadata::MetadataCommand;
use owo_colors::OwoColorize;
use reqwest::Client;
use std::path::PathBuf;
use std::process::Command;

pub async fn check_versions(
    manifest_path: Option<String>,
    _json_output: bool, // Added underscore to silence the warning
    recursive: bool,
    search_path: String,
) -> Result<Vec<(PathBuf, Report)>, VersionError> {
    let manifest_paths = if let Some(path) = manifest_path {
        vec![PathBuf::from(path)]
    } else if recursive {
        find_cargo_files_recursive(&search_path)?
    } else {
        let cargo_files = find_cargo_files(&search_path)?;
        vec![select_cargo_file(cargo_files)?]
    };

    let mut all_reports = Vec::new();

    for manifest_path in &manifest_paths {
        println!("Analyzing {}", manifest_path.display().bold());

        let metadata = get_metadata(manifest_path.to_str().unwrap())?;
        let mut report = Report::new();

        let client = Client::builder()
            .user_agent("cargo-versioncheck/0.1.0")
            .build()?;

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

        for task in tasks {
            if let Ok(Some(dep)) = task.await {
                report.dependencies.push(dep);
            }
        }

        report.dependencies.sort_by(|a, b| a.name.cmp(&b.name));
        report.audit_output = run_cargo_audit()?;
        all_reports.push((manifest_path.clone(), report));
    }

    Ok(all_reports)
}

pub async fn analyze_dependencies(
    manifest_path: Option<String>,
    recursive: bool,
) -> Result<(), VersionError> {
    let manifest_paths = if let Some(path) = manifest_path {
        vec![PathBuf::from(path)]
    } else if recursive {
        find_cargo_files_recursive("..")?
    } else {
        let cargo_files = find_cargo_files("..")?;
        vec![select_cargo_file(cargo_files)?]
    };

    for manifest_path in &manifest_paths {
        println!(
            "Analyzing dependencies in {}",
            manifest_path.display().bold()
        );

        let metadata = get_metadata(manifest_path.to_str().unwrap())?;
        let initial_packages = metadata.packages.clone();

        let status = Command::new("cargo")
            .current_dir(manifest_path.parent().unwrap())
            .arg("update")
            .status()
            .map_err(|e| VersionError::AnalysisError(e.to_string()))?;

        if !status.success() {
            return Err(VersionError::AnalysisError(
                "Failed to update dependencies".into(),
            ));
        }

        let updated_metadata = get_metadata(manifest_path.to_str().unwrap())?;
        let mut analyzer =
            crate::analysis::ImpactAnalyzer::new(manifest_path.parent().unwrap().to_path_buf())
                .map_err(|e| VersionError::AnalysisError(e.to_string()))?;

        let record = analyzer
            .analyze_dependency_updates(&updated_metadata, &initial_packages)
            .map_err(|e| VersionError::AnalysisError(e.to_string()))?;

        if record.dependency_changes.is_empty() {
            println!("{}", "No breaking changes detected.".green().bold());
        } else {
            println!("\n{}", "Breaking Changes Detected:".red().bold());
            println!("{}", "=======================".red());

            for dep in &record.dependency_changes {
                println!(
                    "\n{}: {} → {}",
                    dep.name.yellow().bold(),
                    dep.old_version,
                    dep.new_version
                );
                for change in &dep.breaking_changes {
                    println!("  - {}", change);
                }
            }

            if !record.affected_files.is_empty() {
                println!("\n{}", "Affected Files:".blue().bold());
                println!("{}", "==============".blue());
                for file in &record.affected_files {
                    println!("  {}", file);
                }

                println!("\nDetailed analysis and fix suggestions have been saved to:");
                println!("  .changes/archives/<timestamp>/");
            }
        }
    }

    Ok(())
}

pub fn compare_files(old_file: &str, new_file: &str) -> Result<(), VersionError> {
    println!(
        "Comparing {} with {}",
        old_file.blue().bold(),
        new_file.blue().bold()
    );

    let (old_ast, new_ast) = crate::ast_extractor::extract_asts_for_comparison(old_file, new_file)?;
    let tracker = crate::ast_comparator::compare_asts(&old_ast, &new_ast);

    if tracker.changes.is_empty() {
        println!("{}", "No breaking changes detected.".green().bold());
    } else {
        println!("\n{}", "Breaking Changes Detected:".red().bold());
        println!("{}", "=======================".red());

        for change in tracker.changes {
            match change {
                crate::ast_comparator::BreakingChange::FunctionSignatureChanged {
                    name,
                    old_sig,
                    new_sig,
                } => {
                    println!(
                        "{}: Signature changed\n  From: {}\n    To: {}",
                        name.yellow().bold(),
                        old_sig,
                        new_sig
                    );
                }
                crate::ast_comparator::BreakingChange::StructFieldRemoved {
                    struct_name,
                    field_name,
                } => {
                    println!(
                        "{}: Field removed: {}",
                        struct_name.yellow().bold(),
                        field_name.red()
                    );
                }
                crate::ast_comparator::BreakingChange::StructFieldTypeChanged {
                    struct_name,
                    field_name,
                    old_type,
                    new_type,
                } => {
                    println!(
                        "{}: Field '{}' type changed from {} to {}",
                        struct_name.yellow().bold(),
                        field_name,
                        old_type,
                        new_type
                    );
                }
                crate::ast_comparator::BreakingChange::TraitImplementationRemoved {
                    type_name,
                    trait_name,
                } => {
                    println!(
                        "{}: Removed implementation of trait {}",
                        type_name.yellow().bold(),
                        trait_name.red()
                    );
                }
                crate::ast_comparator::BreakingChange::TraitMethodChanged {
                    trait_name,
                    method_name,
                    details,
                } => {
                    println!(
                        "{}: Method '{}' changed: {}",
                        trait_name.yellow().bold(),
                        method_name,
                        details
                    );
                }
            }
        }
    }

    Ok(())
}

pub fn find_cargo_files_recursive(search_path: &str) -> Result<Vec<PathBuf>, VersionError> {
    let mut cargo_files = Vec::new();
    let walker = walkdir::WalkDir::new(search_path)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            let file_name = e.file_name().to_string_lossy();
            !file_name.starts_with('.') && !file_name.eq("target")
        });

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_name() == "Cargo.toml" {
            cargo_files.push(entry.path().to_path_buf());
        }
    }

    if cargo_files.is_empty() {
        return Err(VersionError::NoCargoFiles);
    }

    Ok(cargo_files)
}

pub fn find_cargo_files(search_path: &str) -> Result<Vec<PathBuf>, VersionError> {
    let mut cargo_files = Vec::new();

    for entry in walkdir::WalkDir::new(search_path)
        .max_depth(1)
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

pub fn select_cargo_file(cargo_files: Vec<PathBuf>) -> Result<PathBuf, VersionError> {
    let options: Vec<String> = cargo_files
        .iter()
        .map(|path| path.display().to_string())
        .collect();

    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select a Cargo.toml file to analyze")
        .items(&options)
        .default(0)
        .interact()
        .map_err(|_| VersionError::SelectionCancelled)?;

    Ok(cargo_files[selection].clone())
}

pub fn get_metadata(manifest_path: &str) -> Result<cargo_metadata::Metadata, VersionError> {
    Ok(MetadataCommand::new().manifest_path(manifest_path).exec()?)
}

async fn get_dependency_info(
    package: &cargo_metadata::Package,
    client: &Client,
) -> Option<DependencyInfo> {
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
            name: name.to_string(),
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
        name: name.to_string(),
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
    let current_version = semver::Version::parse(current_version).ok()?;

    let latest_version = crate_info
        .get("max_version")
        .and_then(|v| v.as_str())
        .and_then(|v| semver::Version::parse(v).ok());

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

pub fn run_cargo_audit() -> Result<String, VersionError> {
    let output = Command::new("cargo").arg("audit").output()?;
    let mut audit_output = String::from_utf8_lossy(&output.stdout).to_string();

    if !output.status.success() {
        audit_output.push_str(&String::from_utf8_lossy(&output.stderr));
    }

    Ok(audit_output)
}
