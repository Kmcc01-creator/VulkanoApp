#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! Version Tracking Tools
//!
//! This crate provides tools for analyzing and generating Rust code with version tracking.
//! It combines AST analysis, code generation, and scripting capabilities to help manage
//! code evolution and API compatibility.
//!
//! # Features
//!
//! - AST analysis for breaking changes
//! - Pattern detection and code generation
//! - Code transformation pipeline
//! - Scripting system for automation
//!
//! # Examples
//!
//! Basic usage:
//! ```rust
//! use versiontracking::{Analyzer, CodeGenerator, GenerationTarget};
//!
//! # fn main() -> anyhow::Result<()> {
//! let analyzer = Analyzer::new();
//! let generator = CodeGenerator::new(Default::default());
//!
//! // Analyze source code
//! let source = r#"
//!     pub fn example() {
//!         println!("test");
//!     }
//! "#;
//! let analysis = analyzer.analyze(&syn::parse_str(source)?);
//!
//! // Generate new code
//! let target = GenerationTarget::Function {
//!     name: "async_example".to_string(),
//!     is_async: true,
//!     is_unsafe: false,
//! };
//!
//! let generated = generator.generate(target)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Feature Flags
//!
//! - `sync`: Synchronous operations (default)
//! - `async`: Async support
//! - `scripting`: Scripting system
//! - `full`: All features

// Re-export from ast-analyzer
pub use ast_analyzer::{
    comparison::ChangeKind, visitor::Visitor, AnalysisResult, Analyzer, BreakingChange, Pattern,
    PatternKind,
};

// Re-export from code-generator
pub use code_generator::{
    patterns::{PatternGenerator, PatternMatch},
    template::{Template, TemplateContext},
    transform::{Transform, TransformConfig},
    CodeGenerator, GenerationTarget, GeneratorConfig,
};

// Internal modules
mod error;

// Scripting module
#[cfg(feature = "scripting")]
#[cfg_attr(docsrs, doc(cfg(feature = "scripting")))]
pub mod scripting;

// Public exports
pub use crate::error::{Error, Result};

/// Convenience function for analyzing code
pub fn analyze(source: &str) -> Result<AnalysisResult> {
    let analyzer = Analyzer::new();
    Ok(analyzer.analyze(&syn::parse_str(source)?))
}

/// Convenience function for code generation
#[cfg(feature = "sync")]
#[cfg_attr(docsrs, doc(cfg(feature = "sync")))]
pub fn generate(target: GenerationTarget) -> Result<proc_macro2::TokenStream> {
    let generator = CodeGenerator::new(Default::default());
    generator.generate(target)
}

/// Async version of code generation
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub async fn generate_async(target: GenerationTarget) -> Result<proc_macro2::TokenStream> {
    let generator = CodeGenerator::new(Default::default());
    generator.generate(target).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_analyze() -> Result<()> {
        let source = r#"
            fn test() {
                println!("test");
            }
        "#;

        let result = analyze(source)?;
        assert!(result.patterns.is_empty());
        Ok(())
    }

    #[cfg(feature = "sync")]
    #[test]
    fn test_generate() -> Result<()> {
        let target = GenerationTarget::Function {
            name: "test".to_string(),
            is_async: false,
            is_unsafe: false,
        };

        let tokens = generate(target)?;
        let code = tokens.to_string();
        assert!(code.contains("fn test"));
        Ok(())
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_generate_async() -> Result<()> {
        let target = GenerationTarget::Function {
            name: "test".to_string(),
            is_async: true,
            is_unsafe: false,
        };

        let tokens = generate_async(target).await?;
        let code = tokens.to_string();
        assert!(code.contains("async fn test"));
        Ok(())
    }

    #[test]
    fn test_breaking_changes() -> Result<()> {
        let old_code: syn::File = parse_quote! {
            pub fn old_function(x: i32) -> i32 {
                x + 1
            }
        };

        let new_code: syn::File = parse_quote! {
            fn old_function(x: i32, y: i32) -> i32 {
                x + y
            }
        };

        let analyzer = Analyzer::new();
        let changes = analyzer.compare_versions(&old_code, &new_code);
        assert!(!changes.is_empty());
        Ok(())
    }
}
