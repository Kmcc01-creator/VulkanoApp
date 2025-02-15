mod analysis;
mod comparison;
mod extraction;
mod pattern;
mod visitor;

use proc_macro2::Span;
use std::collections::HashMap;
use syn::File;

// Re-export main types
pub use crate::comparison::{BreakingChange, ChangeKind};
pub use crate::extraction::{ExtractedField, ExtractedMethod, ExtractedType};
pub use crate::pattern::{PatternDetector, PatternMatch};

/// Main entry point for the AST analyzer
pub struct Analyzer {
    pattern_detector: PatternDetector,
    cache: AnalysisCache,
}

struct AnalysisCache {
    extracted_types: HashMap<String, ExtractedType>,
    breaking_changes: Vec<BreakingChange>,
    patterns: Vec<PatternMatch>,
}

#[derive(Debug)]
pub struct AnalyzerConfig {
    pub detect_unsafe: bool,
    pub detect_blocking: bool,
    pub detect_resources: bool,
    pub check_visibility: bool,
    pub analyze_complexity: bool,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            detect_unsafe: true,
            detect_blocking: true,
            detect_resources: true,
            check_visibility: true,
            analyze_complexity: true,
        }
    }
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            pattern_detector: PatternDetector::new(),
            cache: AnalysisCache {
                extracted_types: HashMap::new(),
                breaking_changes: Vec::new(),
                patterns: Vec::new(),
            },
        }
    }

    pub fn with_config(config: AnalyzerConfig) -> Self {
        // TODO: Configure pattern detector based on config
        Self::new()
    }

    pub fn analyze(&mut self, file: &File) -> AnalysisResult {
        // Extract type information
        let extracted_types = extraction::extract_type_info(&syn::Item::File(file.clone()))
            .into_iter()
            .map(|t| (t.name.clone(), t))
            .collect();

        // Detect patterns
        let patterns = self.pattern_detector.analyze_file(file);

        // Update cache
        self.cache.extracted_types = extracted_types;
        self.cache.patterns = patterns.clone();

        AnalysisResult {
            patterns,
            breaking_changes: Vec::new(), // Only populated when comparing versions
            metrics: AnalysisMetrics::default(),
        }
    }

    pub fn compare_versions(&mut self, old_file: &File, new_file: &File) -> Vec<BreakingChange> {
        let changes = comparison::compare_files(old_file, new_file);
        self.cache.breaking_changes = changes.clone();
        changes
    }

    pub fn get_type_info(&self, name: &str) -> Option<&ExtractedType> {
        self.cache.extracted_types.get(name)
    }

    pub fn get_patterns(&self) -> &[PatternMatch] {
        &self.cache.patterns
    }

    pub fn get_breaking_changes(&self) -> &[BreakingChange] {
        &self.cache.breaking_changes
    }

    pub fn find_pattern(&self, pattern_name: &str) -> Vec<&PatternMatch> {
        self.cache
            .patterns
            .iter()
            .filter(|p| p.pattern == pattern_name)
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct AnalysisMetrics {
    pub function_count: usize,
    pub type_count: usize,
    pub average_complexity: f64,
    pub unsafe_blocks: usize,
    pub blocking_calls: usize,
}

#[derive(Debug)]
pub struct AnalysisResult {
    pub patterns: Vec<PatternMatch>,
    pub breaking_changes: Vec<BreakingChange>,
    pub metrics: AnalysisMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_basic_analysis() {
        let code: File = parse_quote! {
            fn test_function() {
                unsafe {
                    let ptr = std::ptr::null();
                }
            }
        };

        let mut analyzer = Analyzer::new();
        let result = analyzer.analyze(&code);

        assert!(!result.patterns.is_empty());
        assert!(analyzer.find_pattern("unsafe_code").len() > 0);
    }

    #[test]
    fn test_version_comparison() {
        let old_code: File = parse_quote! {
            pub fn old_function() -> i32 { 42 }
        };

        let new_code: File = parse_quote! {
            fn old_function() -> i32 { 42 }
        };

        let mut analyzer = Analyzer::new();
        let changes = analyzer.compare_versions(&old_code, &new_code);

        assert!(!changes.is_empty());
        assert!(matches!(
            changes[0].kind,
            ChangeKind::VisibilityReduced { .. }
        ));
    }

    #[test]
    fn test_type_extraction() {
        let code: File = parse_quote! {
            pub struct TestStruct {
                field: i32,
            }

            impl TestStruct {
                pub fn new() -> Self {
                    Self { field: 0 }
                }
            }
        };

        let mut analyzer = Analyzer::new();
        analyzer.analyze(&code);

        let type_info = analyzer.get_type_info("TestStruct");
        assert!(type_info.is_some());
        assert_eq!(type_info.unwrap().visibility, "pub");
    }
}
