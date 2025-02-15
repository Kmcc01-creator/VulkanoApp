use std::fs;
use std::path::{Path, PathBuf};
use syn::{parse_file, File};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AstError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse Rust source: {0}")]
    ParseError(#[from] syn::Error),

    #[error("Invalid path: {0}")]
    PathError(String),
}

/// Helper function to normalize file paths
fn normalize_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, AstError> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(AstError::PathError(format!(
            "Path does not exist: {}",
            path.display()
        )));
    }
    Ok(fs::canonicalize(path)?)
}

/// Extracts the AST from a given Rust source file.
pub fn extract_ast<P: AsRef<Path>>(path: P) -> Result<File, AstError> {
    let path = normalize_path(path)?;
    let source = fs::read_to_string(path)?;
    Ok(parse_file(&source)?)
}

/// Extracts ASTs from multiple Rust source files in a directory.
#[allow(dead_code)] // Will be used in future expansions
pub fn extract_asts_from_dir<P: AsRef<Path>>(
    dir_path: P,
) -> Result<Vec<(PathBuf, File)>, AstError> {
    let dir_path = normalize_path(dir_path)?;
    if !dir_path.is_dir() {
        return Err(AstError::PathError(format!(
            "Not a directory: {}",
            dir_path.display()
        )));
    }

    let mut asts = Vec::new();
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "rs") {
            match extract_ast(&path) {
                Ok(ast) => asts.push((path, ast)),
                Err(e) => eprintln!("Warning: Failed to parse {}: {}", path.display(), e),
            }
        }
    }

    Ok(asts)
}

/// Extracts ASTs from two versions of a Rust source file for comparison.
pub fn extract_asts_for_comparison<P: AsRef<Path>>(
    old_path: P,
    new_path: P,
) -> Result<(File, File), AstError> {
    let old_ast = extract_ast(old_path)?;
    let new_ast = extract_ast(new_path)?;
    Ok((old_ast, new_ast))
}
