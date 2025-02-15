use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for the scripting system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptConfig {
    /// General settings
    #[serde(default)]
    pub general: GeneralConfig,

    /// Analysis settings
    #[serde(default)]
    pub analysis: AnalysisConfig,

    /// Generation settings
    #[serde(default)]
    pub generation: GenerationConfig,

    /// Pattern settings
    #[serde(default)]
    pub patterns: PatternConfig,
}

/// General configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Script file patterns to include
    pub include: Vec<String>,

    /// Script file patterns to exclude
    pub exclude: Vec<String>,

    /// Path to custom patterns directory
    pub patterns_dir: Option<PathBuf>,

    /// Path to custom templates directory
    pub templates_dir: Option<PathBuf>,

    /// Whether to enable debug output
    pub debug: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            include: vec!["**/*.script".to_string()],
            exclude: vec!["**/test/**".to_string()],
            patterns_dir: None,
            templates_dir: None,
            debug: false,
        }
    }
}

/// Analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Whether to check for safety issues
    pub check_safety: bool,

    /// Whether to detect patterns
    pub detect_patterns: bool,

    /// Whether to be aware of async code
    pub async_aware: bool,

    /// Whether to enforce strict mode
    pub strict: bool,

    /// Custom analysis patterns to enable
    pub custom_patterns: Vec<String>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            check_safety: true,
            detect_patterns: true,
            async_aware: true,
            strict: false,
            custom_patterns: Vec::new(),
        }
    }
}

/// Code generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    /// Whether to generate documentation
    pub documentation: bool,

    /// Code optimization level
    pub optimization: OptimizationLevel,

    /// Whether to add safety checks
    pub safety_checks: bool,

    /// Default visibility for generated items
    pub default_visibility: String,

    /// Custom transforms to apply
    pub custom_transforms: Vec<String>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            documentation: true,
            optimization: OptimizationLevel::Default,
            safety_checks: true,
            default_visibility: "pub".to_string(),
            custom_transforms: Vec::new(),
        }
    }
}

/// Code optimization level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OptimizationLevel {
    /// No optimization
    None,
    /// Default optimization
    Default,
    /// Optimize for size
    Size,
    /// Optimize for speed
    Speed,
}

/// Pattern configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConfig {
    /// Default pattern options
    pub defaults: HashMap<String, String>,

    /// Pattern-specific configurations
    pub overrides: HashMap<String, PatternOverride>,
}

impl Default for PatternConfig {
    fn default() -> Self {
        Self {
            defaults: HashMap::new(),
            overrides: HashMap::new(),
        }
    }
}

/// Pattern-specific configuration override
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternOverride {
    /// Whether the pattern is enabled
    pub enabled: bool,

    /// Pattern-specific options
    pub options: HashMap<String, String>,
}

impl ScriptConfig {
    /// Loads configuration from a file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Saves configuration to a file
    pub fn save_to_file(&self, path: impl AsRef<std::path::Path>) -> Result<(), std::io::Error> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, content)
    }

    /// Creates a default configuration
    pub fn default_config() -> Self {
        Self {
            general: GeneralConfig::default(),
            analysis: AnalysisConfig::default(),
            generation: GenerationConfig::default(),
            patterns: PatternConfig::default(),
        }
    }

    /// Gets a pattern configuration
    pub fn get_pattern_config(&self, name: &str) -> HashMap<String, String> {
        let mut config = self.patterns.defaults.clone();

        if let Some(override_config) = self.patterns.overrides.get(name) {
            if override_config.enabled {
                config.extend(override_config.options.clone());
            }
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ScriptConfig::default_config();
        assert!(config.analysis.check_safety);
        assert!(config.generation.documentation);
    }

    #[test]
    fn test_pattern_config() {
        let mut config = ScriptConfig::default_config();

        // Add default pattern config
        config
            .patterns
            .defaults
            .insert("async".to_string(), "true".to_string());

        // Add override
        let mut override_config = PatternOverride {
            enabled: true,
            options: HashMap::new(),
        };
        override_config
            .options
            .insert("async".to_string(), "false".to_string());

        config
            .patterns
            .overrides
            .insert("test_pattern".to_string(), override_config);

        // Check override takes precedence
        let pattern_config = config.get_pattern_config("test_pattern");
        assert_eq!(pattern_config.get("async").unwrap(), "false");
    }

    #[test]
    fn test_serialization() {
        let config = ScriptConfig::default_config();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: ScriptConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(
            config.analysis.check_safety,
            deserialized.analysis.check_safety
        );
    }
}
