use anyhow::Result;
use std::path::PathBuf;
use tempfile::TempDir;
use versiontracking::scripting::ScriptConfig;

/// Test setup helper that manages temporary directories and configurations
pub struct TestSetup {
    /// Temporary directory for test files
    pub temp_dir: TempDir,
    /// Configuration for the test
    pub config: ScriptConfig,
}

impl TestSetup {
    /// Creates a new test setup with temporary directory and default configuration
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().expect("Failed to create temp directory"),
            config: ScriptConfig::default_config(),
        }
    }

    /// Creates test source file with given content
    pub fn create_source_file(&self, name: &str, content: &str) -> Result<PathBuf> {
        let path = self.temp_dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        Ok(path)
    }

    /// Creates test configuration file
    pub fn create_config_file(&self) -> Result<PathBuf> {
        let path = self.temp_dir.path().join(".version-tracking.toml");
        self.config.save_to_file(&path)?;
        Ok(path)
    }

    /// Creates a project structure for testing
    pub fn create_project(&self) -> Result<ProjectSetup> {
        let src_dir = self.temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir)?;

        // Create basic project structure
        self.create_source_file(
            "src/lib.rs",
            r#"
                pub mod example;
                pub use example::Example;
            "#,
        )?;

        self.create_source_file(
            "src/example.rs",
            r#"
                pub struct Example {
                    field: String,
                }

                impl Example {
                    pub fn new(field: String) -> Self {
                        Self { field }
                    }
                }
            "#,
        )?;

        Ok(ProjectSetup {
            root: self.temp_dir.path().to_path_buf(),
            src_dir,
        })
    }
}

impl Default for TestSetup {
    fn default() -> Self {
        Self::new()
    }
}

/// Project structure for tests
pub struct ProjectSetup {
    /// Root directory of the test project
    pub root: PathBuf,
    /// Source directory of the test project
    pub src_dir: PathBuf,
}

impl ProjectSetup {
    /// Creates a new file in the project
    pub fn create_file(&self, name: &str, content: &str) -> Result<PathBuf> {
        let path = self.src_dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        Ok(path)
    }

    /// Creates a module in the project
    pub fn create_module(&self, name: &str, content: &str) -> Result<PathBuf> {
        self.create_file(&format!("{}.rs", name), content)
    }
}

/// Test utilities for code comparison
pub mod assert {
    use similar_asserts::assert_eq;

    /// Asserts that two strings of code are semantically equivalent
    pub fn assert_code_eq(actual: &str, expected: &str) {
        let actual = normalize_code(actual);
        let expected = normalize_code(expected);
        assert_eq!(actual, expected);
    }

    /// Normalizes code by removing whitespace and comments
    fn normalize_code(code: &str) -> String {
        code.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_creation() -> Result<()> {
        let setup = TestSetup::new();
        let path = setup.create_source_file("test.rs", "fn test() {}")?;
        assert!(path.exists());
        Ok(())
    }

    #[test]
    fn test_project_creation() -> Result<()> {
        let setup = TestSetup::new();
        let project = setup.create_project()?;
        assert!(project.src_dir.exists());
        assert!(project.src_dir.join("lib.rs").exists());
        Ok(())
    }

    #[test]
    fn test_code_comparison() {
        assert::assert_code_eq(
            "fn test() { println!(\"test\"); }",
            "fn test() {
                println!(\"test\");
            }",
        );
    }
}
