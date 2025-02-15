//! Script interpreter for executing commands

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{Command, Context};
use crate::{
    Analyzer, CodeGenerator, GenerationTarget, GeneratorConfig, Pattern, Transform, TransformConfig,
};

/// Script interpreter for executing commands
pub struct Interpreter {
    context: Arc<RwLock<Context>>,
    analyzer: Analyzer,
    generator: CodeGenerator,
}

impl Interpreter {
    /// Creates a new interpreter
    pub fn new() -> Self {
        Self {
            context: Arc::new(RwLock::new(Context::new())),
            analyzer: Analyzer::new(),
            generator: CodeGenerator::new(Default::default()),
        }
    }

    /// Executes a script
    pub async fn execute_script(&mut self, script: &str) -> Result<()> {
        let commands = super::parser::parse_script(script)?;

        for command in commands {
            self.execute_command(command).await?;
        }

        Ok(())
    }

    /// Executes a single command
    pub async fn execute_command(&mut self, command: Command) -> Result<()> {
        match command {
            Command::ApplyPattern {
                name,
                target,
                options,
            } => {
                self.execute_pattern(&name, &target, options).await?;
            }

            Command::Analyze {
                target,
                patterns,
                options,
            } => {
                self.execute_analysis(&target, patterns, options).await?;
            }

            Command::Generate {
                name,
                kind,
                fields,
                options,
            } => {
                self.execute_generation(&name, &kind, fields, options)
                    .await?;
            }

            Command::Configure { section, settings } => {
                self.apply_configuration(&section, settings).await?;
            }
        }

        Ok(())
    }

    async fn execute_pattern(
        &mut self,
        name: &str,
        target: &str,
        options: std::collections::HashMap<String, String>,
    ) -> Result<()> {
        let context = self.context.read().await;
        let pattern = context
            .patterns
            .get(name)
            .ok_or_else(|| crate::Error::pattern(format!("Pattern not found: {}", name)))?;

        // Create generation context
        let mut template_context = crate::TemplateContext::new();
        for (k, v) in options {
            template_context.set(k, v);
        }

        // Generate code from pattern
        let code = pattern.generate(&template_context).await?;
        println!("Generated code for pattern '{}':\n{}", name, code);

        Ok(())
    }

    async fn execute_analysis(
        &mut self,
        target: &str,
        patterns: Vec<String>,
        options: std::collections::HashMap<String, bool>,
    ) -> Result<()> {
        // Read the target file
        let source = std::fs::read_to_string(target)?;
        let ast = syn::parse_str(&source)?;

        // Configure analyzer based on options
        if let Some(&check_safety) = options.get("check_safety") {
            // Configure safety checks
        }

        // Perform analysis
        let analysis = self.analyzer.analyze(&ast);

        // Filter for requested patterns
        for pattern_name in patterns {
            let matches = analysis
                .patterns
                .iter()
                .filter(|p| p.pattern == pattern_name)
                .collect::<Vec<_>>();

            if !matches.is_empty() {
                println!(
                    "Found {} instances of pattern '{}':",
                    matches.len(),
                    pattern_name
                );
                for pattern in matches {
                    println!("  - {}", pattern.context);
                }
            }
        }

        Ok(())
    }

    async fn execute_generation(
        &mut self,
        name: &str,
        kind: &str,
        fields: Vec<String>,
        options: std::collections::HashMap<String, String>,
    ) -> Result<()> {
        let target = match kind {
            "struct" => GenerationTarget::Struct {
                name: name.to_string(),
                fields,
            },
            "function" => GenerationTarget::Function {
                name: name.to_string(),
                is_async: options.get("async").map_or(false, |v| v == "true"),
                is_unsafe: options.get("unsafe").map_or(false, |v| v == "true"),
            },
            "trait" => GenerationTarget::Trait {
                name: name.to_string(),
                methods: fields,
            },
            _ => {
                return Err(
                    crate::Error::other(format!("Unknown generation kind: {}", kind)).into(),
                )
            }
        };

        // Configure generator
        let config = GeneratorConfig {
            documentation: options.get("doc").map_or(true, |v| v == "true"),
            ..Default::default()
        };

        self.generator = CodeGenerator::new(config);

        // Generate code
        let code = self.generator.generate(target).await?;
        println!("Generated {}:\n{}", kind, code);

        Ok(())
    }

    async fn apply_configuration(
        &mut self,
        section: &str,
        settings: std::collections::HashMap<String, String>,
    ) -> Result<()> {
        match section {
            "analysis" => {
                // Configure analyzer
                for (key, value) in settings {
                    match key.as_str() {
                        "check_safety" => {
                            if value == "true" {
                                // Enable safety checks
                            }
                        }
                        _ => println!("Unknown analysis setting: {}", key),
                    }
                }
            }
            "generation" => {
                // Configure generator
                let mut config = GeneratorConfig::default();
                for (key, value) in settings {
                    match key.as_str() {
                        "documentation" => {
                            config.documentation = value == "true";
                        }
                        "transform" => {
                            config.transform_config = TransformConfig {
                                async_conversion: value == "async",
                                ..Default::default()
                            };
                        }
                        _ => println!("Unknown generation setting: {}", key),
                    }
                }
                self.generator = CodeGenerator::new(config);
            }
            _ => println!("Unknown configuration section: {}", section),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pattern_execution() -> Result<()> {
        let mut interpreter = Interpreter::new();

        let script = r#"
            apply_pattern builder TestStruct {
                visibility = "pub"
                async = "true"
            }
        "#;

        interpreter.execute_script(script).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_analysis_execution() -> Result<()> {
        let mut interpreter = Interpreter::new();

        let script = r#"
            analyze src/lib.rs {
                detect unsafe_blocks
                check_safety = true
            }
        "#;

        interpreter.execute_script(script).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_configuration() -> Result<()> {
        let mut interpreter = Interpreter::new();

        let script = r#"
            configure generation {
                documentation = "true"
                transform = "async"
            }
        "#;

        interpreter.execute_script(script).await?;
        Ok(())
    }
}
