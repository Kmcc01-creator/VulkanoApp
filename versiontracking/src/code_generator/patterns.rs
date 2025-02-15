use crate::code_generator::types::{CodeLocation, PatternMatch, PatternType, Suggestion};
use quote::ToTokens;
use std::collections::HashMap;
use syn::{visit::Visit, Block, Expr, ExprAsync, ExprAwait, ExprCall, ItemFn, Type};

/// Analyzes Rust source code for patterns and anti-patterns.
pub struct PatternAnalyzer {
    patterns: Vec<PatternMatch>,
    current_file: Option<std::path::PathBuf>,
    block_stack: Vec<BlockContext>,
    locks_in_scope: Vec<String>,
    potential_clones: HashMap<String, usize>,
}

#[derive(Default)]
struct BlockContext {
    is_async: bool,
    contains_blocking: bool,
    lock_count: usize,
    heap_allocs: usize,
}

impl PatternAnalyzer {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            current_file: None,
            block_stack: Vec::new(),
            locks_in_scope: Vec::new(),
            potential_clones: HashMap::new(),
        }
    }

    pub fn set_current_file(&mut self, file: std::path::PathBuf) {
        self.current_file = Some(file);
    }

    pub fn analyze_ast(&mut self, ast: &syn::File) -> Vec<PatternMatch> {
        self.visit_file(ast);
        std::mem::take(&mut self.patterns)
    }

    fn add_pattern(&mut self, pattern_type: PatternType, line: usize, context: String) {
        if let Some(file) = &self.current_file {
            self.patterns.push(PatternMatch {
                pattern_type: pattern_type.clone(),
                location: CodeLocation {
                    file: file.clone(),
                    start_line: line,
                    end_line: line + 1,
                    context,
                },
                suggestions: self.generate_suggestions(pattern_type),
            });
        }
    }

    fn generate_suggestions(&self, pattern_type: PatternType) -> Vec<Suggestion> {
        match pattern_type {
            PatternType::AsyncAntiPattern => vec![Suggestion {
                title: "Convert to async".to_string(),
                description: "Convert blocking operation to async/await".to_string(),
                code: "async fn example() { let result = some_async_op().await; }".to_string(),
                safety_impact: None,
                performance_impact: Some("Improves responsiveness".to_string()),
            }],
            PatternType::LockContention => vec![Suggestion {
                title: "Reduce lock scope".to_string(),
                description: "Minimize the critical section".to_string(),
                code: "{ let data = mutex.lock().unwrap(); // Minimal work here }".to_string(),
                safety_impact: None,
                performance_impact: Some("Reduces contention".to_string()),
            }],
            PatternType::VecWithCapacity => vec![Suggestion {
                title: "Use with_capacity".to_string(),
                description: "Pre-allocate vector capacity".to_string(),
                code: "Vec::with_capacity(expected_size)".to_string(),
                safety_impact: None,
                performance_impact: Some("Reduces reallocations".to_string()),
            }],
            _ => Vec::new(),
        }
    }

    fn analyze_blocking_calls(&self, expr: &ExprCall) -> bool {
        let known_blocking = [
            "read_to_string",
            "write_all",
            "sleep",
            "join",
            "connect",
            "lock",
        ];

        if let Expr::Path(path) = &*expr.func {
            let func_name = path
                .path
                .segments
                .last()
                .map(|seg| seg.ident.to_string())
                .unwrap_or_default();
            known_blocking.contains(&func_name.as_str())
        } else {
            false
        }
    }

    fn analyze_clone_necessity(&self, expr: &ExprCall, context: &Block) -> bool {
        // TODO: Implement more sophisticated clone analysis
        false
    }

    fn analyze_lock_patterns(&mut self, _block: &Block) {
        // Track nested locks
        if self.block_stack.last().map_or(0, |ctx| ctx.lock_count) > 1 {
            self.add_pattern(
                PatternType::LockContention,
                0, // TODO: Get proper line numbers
                "Multiple locks held simultaneously".to_string(),
            );
        }
    }

    fn analyze_allocation_patterns(&mut self, expr: &Expr) {
        if let Expr::Call(call) = expr {
            if let Expr::Path(path) = &*call.func {
                let func_name = path
                    .path
                    .segments
                    .last()
                    .map(|seg| seg.ident.to_string())
                    .unwrap_or_default();

                if func_name == "Vec::new" || func_name == "String::new" {
                    if let Some(context) = self.block_stack.last() {
                        if context.heap_allocs > 5 {
                            self.add_pattern(
                                PatternType::VecWithCapacity,
                                0, // TODO: Get proper line numbers
                                "Consider using with_capacity for better performance".to_string(),
                            );
                        }
                    }
                }
            }
        }
    }

    fn analyze_async_patterns(&mut self, func: &ItemFn) {
        if func.sig.asyncness.is_none() {
            let mut has_blocking = false;
            let mut has_await = false;

            struct AsyncPatternVisitor {
                has_blocking: bool,
                has_await: bool,
            }

            impl<'ast> Visit<'ast> for AsyncPatternVisitor {
                fn visit_expr_call(&mut self, call: &'ast ExprCall) {
                    if let Expr::Path(path) = &*call.func {
                        let func_name = path
                            .path
                            .segments
                            .last()
                            .map(|seg| seg.ident.to_string())
                            .unwrap_or_default();
                        if ["sleep", "read_to_string", "write", "lock"]
                            .contains(&func_name.as_str())
                        {
                            self.has_blocking = true;
                        }
                    }
                    syn::visit::visit_expr_call(self, call);
                }

                fn visit_expr_await(&mut self, _await: &'ast ExprAwait) {
                    self.has_await = true;
                }

                fn visit_expr_async(&mut self, _async: &'ast ExprAsync) {
                    self.has_await = true;
                }
            }

            let mut visitor = AsyncPatternVisitor {
                has_blocking: false,
                has_await: false,
            };
            visitor.visit_item_fn(func);

            if visitor.has_blocking && !visitor.has_await {
                self.add_pattern(
                    PatternType::AsyncAntiPattern,
                    0, // TODO: Get proper line numbers
                    "Function contains blocking calls but is not async".to_string(),
                );
            }
        }
    }
}

impl<'ast> Visit<'ast> for PatternAnalyzer {
    fn visit_block(&mut self, block: &'ast Block) {
        self.block_stack.push(BlockContext::default());
        self.analyze_lock_patterns(block);
        syn::visit::visit_block(self, block);
        self.block_stack.pop();
    }

    fn visit_item_fn(&mut self, func: &'ast ItemFn) {
        self.analyze_async_patterns(func);
        syn::visit::visit_item_fn(self, func);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        self.analyze_allocation_patterns(expr);

        if let Expr::Call(call) = expr {
            if self.analyze_blocking_calls(call) {
                if let Some(context) = self.block_stack.last() {
                    if context.is_async {
                        self.add_pattern(
                            PatternType::AsyncAntiPattern,
                            0, // TODO: Get proper line numbers
                            "Blocking call in async context".to_string(),
                        );
                    }
                }
            }
        }

        syn::visit::visit_expr(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn test_detect_async_anti_pattern() {
        let code = r#"
            fn blocking_function() {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        "#;

        let ast = parse_str(code).unwrap();
        let mut analyzer = PatternAnalyzer::new();
        analyzer.set_current_file(std::path::PathBuf::from("test.rs"));
        let patterns = analyzer.analyze_ast(&ast);

        assert!(patterns
            .iter()
            .any(|p| matches!(p.pattern_type, PatternType::AsyncAntiPattern)));
    }

    #[test]
    fn test_detect_lock_contention() {
        let code = r#"
            fn complex_locks() {
                let _lock1 = mutex1.lock().unwrap();
                let _lock2 = mutex2.lock().unwrap();
            }
        "#;

        let ast = parse_str(code).unwrap();
        let mut analyzer = PatternAnalyzer::new();
        analyzer.set_current_file(std::path::PathBuf::from("test.rs"));
        let patterns = analyzer.analyze_ast(&ast);

        assert!(patterns
            .iter()
            .any(|p| matches!(p.pattern_type, PatternType::LockContention)));
    }
}
