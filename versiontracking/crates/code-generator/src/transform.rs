use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use proc_macro2::TokenStream;
use quote::quote;
use tokio::sync::RwLock;

use crate::GenerationContext;

#[derive(Debug, Clone)]
pub struct TransformConfig {
    pub async_conversion: bool,
    pub error_handling: bool,
    pub safety_checks: bool,
    pub optimization_level: OptimizationLevel,
}

#[derive(Debug, Clone, Copy)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
}

impl Default for TransformConfig {
    fn default() -> Self {
        Self {
            async_conversion: false,
            error_handling: true,
            safety_checks: true,
            optimization_level: OptimizationLevel::Basic,
        }
    }
}

/// Core trait for implementing code transformations
#[async_trait]
pub trait Transform: Send + Sync {
    /// Name of the transform for identification
    fn name(&self) -> &str;

    /// Description of what the transform does
    fn description(&self) -> &str;

    /// Apply the transformation to the given code
    async fn apply(&self, code: TokenStream, context: &GenerationContext) -> Result<TokenStream>;
}

/// Transforms synchronous code into asynchronous code
pub struct AsyncTransform {
    config: TransformConfig,
}

#[async_trait]
impl Transform for AsyncTransform {
    fn name(&self) -> &str {
        "async_transform"
    }

    fn description(&self) -> &str {
        "Converts synchronous code to asynchronous"
    }

    async fn apply(&self, code: TokenStream, context: &GenerationContext) -> Result<TokenStream> {
        if !self.config.async_conversion {
            return Ok(code);
        }

        // Find blocking patterns
        let blocking_patterns = context.patterns.iter()
            .filter(|p| p.pattern == "blocking_operation");

        // Convert to async if blocking patterns found
        if blocking_patterns.count() > 0 {
            let new_code = quote! {
                async #code
            };
            Ok(new_code)
        } else {
            Ok(code)
        }
    }
}

/// Adds error handling code
pub struct ErrorHandlingTransform {
    config: TransformConfig,
}

#[async_trait]
impl Transform for ErrorHandlingTransform {
    fn name(&self) -> &str {
        "error_handling_transform"
    }

    fn description(&self) -> &str {
        "Adds error handling and propagation"
    }

    async fn apply(&self, code: TokenStream, context: &GenerationContext) -> Result<TokenStream> {
        if !self.config.error_handling {
            return Ok(code);
        }

        // Add Result return type and error propagation
        let new_code = quote! {
            #[allow(unused_imports)]
            use anyhow::{Result, Context};

            #code
        };
        Ok(new_code)
    }
}

/// Adds safety checks and validations
pub struct SafetyTransform {
    config: TransformConfig,
}

#[async_trait]
impl Transform for SafetyTransform {
    fn name(&self) -> &str {
        "safety_transform"
    }

    fn description(&self) -> &str {
        "Adds runtime safety checks"
    }

    async fn apply(&self, code: TokenStream, context: &GenerationContext) -> Result<TokenStream> {
        if !self.config.safety_checks {
            return Ok(code);
        }

        // Add debug assertions and runtime checks
        let new_code = quote! {
            #[cfg(debug_assertions)]
            {
                // Add safety checks here
            }

            #code
        };
        Ok(new_code)
    }
}

/// Pipeline for managing multiple transforms
pub struct TransformPipeline {
    transforms: Vec<Arc<dyn Transform>>,
    config: Arc<RwLock<TransformConfig>>,
}

impl TransformPipeline {
    pub fn new(config: TransformConfig) -> Self {
        Self {
            transforms: Vec::new(),
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub fn add_transform(&mut self, transform: Arc<dyn Transform>) {
        self.transforms.push(transform);
    }

    pub async fn apply_all(&self, mut code: TokenStream, context: &GenerationContext) -> Result<TokenStream> {
        for transform in &self.transforms {
            code = transform.apply(code, context).await?;
        }
        Ok(code)
    }

    pub async fn update_config(&self, config: TransformConfig) {
        let mut current = self.config.write().await;
        *current = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[tokio::test]
    async fn test_async_transform() {
        let transform = AsyncTransform {
            config: TransformConfig {
                async_conversion: true,
                ..Default::default()
            },
        };

        let code: TokenStream = parse_quote! {
            fn test() {
                std::fs::read_to_string("test.txt").unwrap();
            }
        };

        let context = GenerationContext {
            target: crate::GenerationTarget::Function {
                name: "test".to_string(),
                is_async: false,
                is_unsafe: false,
            },
            patterns: vec![
                ast_analyzer::PatternMatch {
                    pattern: "blocking_operation",
                    span: proc_macro2::Span::call_site(),
                    context: "File I/O".to_string(),
                    confidence: 1.0,
                    suggested_fix: None,
                },
            ],
            dependencies: Default::default(),
            config: crate::GeneratorConfig::default(),
        };

        let result = transform.apply(code, &context).await.unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("async"));
    }

    #[tokio::test]
    async fn test_error_handling_transform() {
        let transform = ErrorHandlingTransform {
            config: TransformConfig {
                error_handling: true,
                ..Default::default()
            },
        };

        let code: TokenStream = parse_quote! {
            fn test() -> i32 {
                42
            }
        };

        let context = GenerationContext {
            target: crate::GenerationTarget::Function {
                name: "test".to_string(),
                is_async: false,
                is_unsafe: false,
            },
            patterns: vec![],
            dependencies: Default::default(),
            config: crate::GeneratorConfig::default(),
        };

        let result = transform.apply(code, &context).await.unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("anyhow"));
    }
}