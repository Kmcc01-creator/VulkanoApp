mod analysis;
mod comparison;
mod extraction;
mod pattern;
mod visitor;

use proc_macro2::Span;
use std::collections::HashMap;
use syn::{File, Item, Signature};

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub breaking_changes: Vec<BreakingChange>,
    pub patterns: Vec<Pattern>,
    pub metrics: AnalysisMetrics,
}

#[derive(Debug, Clone)]
pub struct BreakingChange {
    pub kind: ChangeKind,
    pub span: Span,
    pub description: String,
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ChangeKind {
    SignatureChanged {
        old_sig: String,
        new_sig: String,
    },
    TypeChanged {
        path: String,
        old_type: String,
        new_type: String,
    },
    TraitImplementationRemoved {
        trait_name: String,
        type_name: String,
    },
    MethodRemoved {
        method_name: String,
        type_name: String,
    },
    VisibilityReduced {
        item_name: String,
        old_vis: String,
        new_vis: String,
    },
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub kind: PatternKind,
    pub span: Span,
    pub context: String,
}

#[derive(Debug, Clone)]
pub enum PatternKind {
    BlockingCall,
    UnsafeBlock,
    ResourceLeak,
    ConcurrencyIssue,
    PerformanceCritical,
    SecuritySensitive,
}

#[derive(Debug, Clone, Default)]
pub struct AnalysisMetrics {
    pub function_count: usize,
    pub struct_count: usize,
    pub trait_count: usize,
    pub unsafe_block_count: usize,
    pub blocking_call_count: usize,
    pub complexity_score: f64,
}

/// Core analyzer that coordinates different analysis aspects
pub struct Analyzer {
    patterns: HashMap<String, Box<dyn PatternMatcher>>,
    metrics: AnalysisMetrics,
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            metrics: AnalysisMetrics::default(),
        }
    }

    pub fn register_pattern(&mut self, name: String, matcher: Box<dyn PatternMatcher>) {
        self.patterns.insert(name, matcher);
    }

    pub fn analyze_file(&mut self, file: &File) -> AnalysisResult {
        let mut visitor = visitor::AstVisitor::new();
        visitor.visit_file(file);

        let mut patterns = Vec::new();
        for matcher in self.patterns.values() {
            patterns.extend(matcher.find_matches(file));
        }

        AnalysisResult {
            breaking_changes: Vec::new(), // Will be populated when comparing versions
            patterns,
            metrics: self.metrics.clone(),
        }
    }

    pub fn compare_files(&self, old_file: &File, new_file: &File) -> Vec<BreakingChange> {
        comparison::compare_files(old_file, new_file)
    }

    pub fn extract_signatures(&self, file: &File) -> HashMap<String, Signature> {
        extraction::extract_signatures(file)
    }
}

pub trait PatternMatcher: Send + Sync {
    fn find_matches(&self, file: &File) -> Vec<Pattern>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

#[derive(Debug)]
pub struct AnalysisError {
    pub kind: AnalysisErrorKind,
    pub span: Option<Span>,
    pub message: String,
}

#[derive(Debug)]
pub enum AnalysisErrorKind {
    ParseError,
    ValidationError,
    PatternMatchError,
    ComparisonError,
}

impl std::error::Error for AnalysisError {}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    struct TestPattern;

    impl PatternMatcher for TestPattern {
        fn find_matches(&self, _file: &File) -> Vec<Pattern> {
            vec![]
        }

        fn name(&self) -> &str {
            "test_pattern"
        }

        fn description(&self) -> &str {
            "Test pattern for unit tests"
        }
    }

    #[test]
    fn test_analyzer_initialization() {
        let analyzer = Analyzer::new();
        assert_eq!(analyzer.patterns.len(), 0);
    }

    #[test]
    fn test_pattern_registration() {
        let mut analyzer = Analyzer::new();
        analyzer.register_pattern("test".to_string(), Box::new(TestPattern));
        assert_eq!(analyzer.patterns.len(), 1);
    }

    #[test]
    fn test_basic_analysis() {
        let mut analyzer = Analyzer::new();
        let file: File = parse_quote! {
            fn test_function() -> i32 {
                42
            }
        };

        let result = analyzer.analyze_file(&file);
        assert!(result.breaking_changes.is_empty());
    }
}
