use proc_macro2::Span;
use std::collections::HashMap;
use syn::{
    visit::Visit, Expr, ExprCall, ExprLoop, ExprMatch, ExprMethodCall, ExprTry, ExprUnsafe, File,
    ImplItem, ItemImpl, ItemMod, ItemTrait,
};

#[derive(Debug)]
pub struct PatternMatch {
    pub pattern: &'static str,
    pub span: Span,
    pub context: String,
    pub confidence: f32,
    pub suggested_fix: Option<String>,
}

pub struct PatternDetector {
    matches: Vec<PatternMatch>,
    statistics: DetectionStatistics,
}

#[derive(Default)]
struct DetectionStatistics {
    unsafe_blocks: usize,
    blocking_calls: usize,
    concurrent_access: usize,
    resource_leaks: usize,
    error_handling: usize,
}

impl PatternDetector {
    pub fn new() -> Self {
        Self {
            matches: Vec::new(),
            statistics: DetectionStatistics::default(),
        }
    }

    pub fn analyze_file(&mut self, file: &File) -> Vec<PatternMatch> {
        let mut visitor = PatternVisitor::new();
        visitor.visit_file(file);

        // Process unsafe patterns
        for span in visitor.unsafe_blocks {
            self.matches.push(PatternMatch {
                pattern: "unsafe_code",
                span,
                context: "Unsafe block detected".to_string(),
                confidence: 1.0,
                suggested_fix: Some("Consider using safe abstractions".to_string()),
            });
            self.statistics.unsafe_blocks += 1;
        }

        // Process blocking calls
        for (span, name) in visitor.blocking_calls {
            self.matches.push(PatternMatch {
                pattern: "blocking_operation",
                span,
                context: format!("Blocking call detected: {}", name),
                confidence: 0.9,
                suggested_fix: Some("Consider using async alternatives".to_string()),
            });
            self.statistics.blocking_calls += 1;
        }

        // Process resource patterns
        for (span, resource) in visitor.resource_usage {
            self.matches.push(PatternMatch {
                pattern: "resource_management",
                span,
                context: format!("Resource usage: {}", resource),
                confidence: 0.8,
                suggested_fix: Some("Ensure proper resource cleanup".to_string()),
            });
            self.statistics.resource_leaks += 1;
        }

        // Process error handling
        for (span, context) in visitor.error_handling {
            self.matches.push(PatternMatch {
                pattern: "error_handling",
                span,
                context,
                confidence: 0.85,
                suggested_fix: Some("Consider using anyhow/thiserror".to_string()),
            });
            self.statistics.error_handling += 1;
        }

        self.matches.clone()
    }

    pub fn get_statistics(&self) -> &DetectionStatistics {
        &self.statistics
    }
}

struct PatternVisitor {
    unsafe_blocks: Vec<Span>,
    blocking_calls: Vec<(Span, String)>,
    resource_usage: Vec<(Span, String)>,
    error_handling: Vec<(Span, String)>,
    in_async_context: bool,
}

impl PatternVisitor {
    fn new() -> Self {
        Self {
            unsafe_blocks: Vec::new(),
            blocking_calls: Vec::new(),
            resource_usage: Vec::new(),
            error_handling: Vec::new(),
            in_async_context: false,
        }
    }

    fn is_blocking_call(&self, path: &str) -> bool {
        path.contains("fs::")
            || path.contains("net::")
            || path.contains("thread::sleep")
            || path.contains("stdin")
            || path.contains("stdout")
    }

    fn is_resource_type(&self, ty: &str) -> bool {
        ty.contains("File")
            || ty.contains("Socket")
            || ty.contains("Connection")
            || ty.contains("Mutex")
            || ty.contains("Lock")
    }
}

impl<'ast> Visit<'ast> for PatternVisitor {
    fn visit_expr_unsafe(&mut self, expr: &'ast ExprUnsafe) {
        self.unsafe_blocks.push(expr.unsafe_token.span);
        syn::visit::visit_expr_unsafe(self, expr);
    }

    fn visit_expr_call(&mut self, expr: &'ast ExprCall) {
        if let Expr::Path(path) = &*expr.func {
            let path_str = path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if self.is_blocking_call(&path_str) {
                self.blocking_calls.push((expr.func.span(), path_str));
            }
        }
        syn::visit::visit_expr_call(self, expr);
    }

    fn visit_expr_method_call(&mut self, expr: &'ast ExprMethodCall) {
        let method_name = expr.method.to_string();
        if method_name.starts_with("lock") || method_name.starts_with("blocking_") {
            self.blocking_calls.push((expr.method.span(), method_name));
        }
        syn::visit::visit_expr_method_call(self, expr);
    }

    fn visit_expr_try(&mut self, expr: &'ast ExprTry) {
        self.error_handling.push((
            expr.question_token.span,
            "Error propagation with ?".to_string(),
        ));
        syn::visit::visit_expr_try(self, expr);
    }

    fn visit_expr_match(&mut self, expr: &'ast ExprMatch) {
        if let Expr::Try(_) = &*expr.expr {
            self.error_handling
                .push((expr.expr.span(), "Match on Result/Option".to_string()));
        }
        syn::visit::visit_expr_match(self, expr);
    }

    fn visit_expr_loop(&mut self, expr: &'ast ExprLoop) {
        // Check for potential infinite loops
        if !expr.label.is_some() {
            self.blocking_calls
                .push((expr.loop_token.span, "Potential infinite loop".to_string()));
        }
        syn::visit::visit_expr_loop(self, expr);
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        // Check for Drop implementation
        if let Some((_, path, _)) = &item.trait_ {
            if path
                .segments
                .last()
                .map(|s| s.ident == "Drop")
                .unwrap_or(false)
            {
                self.resource_usage
                    .push((item.self_ty.span(), "Resource cleanup in Drop".to_string()));
            }
        }
        syn::visit::visit_item_impl(self, item);
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        // Look for async traits
        for item in &item.items {
            if let syn::TraitItem::Fn(method) = item {
                if method.sig.asyncness.is_some() {
                    self.in_async_context = true;
                }
            }
        }
        syn::visit::visit_item_trait(self, item);
        self.in_async_context = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_detect_unsafe() {
        let code: File = parse_quote! {
            fn test() {
                unsafe {
                    let ptr = std::ptr::null();
                }
            }
        };

        let mut detector = PatternDetector::new();
        let matches = detector.analyze_file(&code);

        assert!(!matches.is_empty());
        assert_eq!(matches[0].pattern, "unsafe_code");
    }

    #[test]
    fn test_detect_blocking() {
        let code: File = parse_quote! {
            fn test() {
                std::fs::read_to_string("test.txt").unwrap();
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        };

        let mut detector = PatternDetector::new();
        let matches = detector.analyze_file(&code);

        assert!(matches.iter().any(|m| m.pattern == "blocking_operation"));
    }

    #[test]
    fn test_detect_resource_management() {
        let code: File = parse_quote! {
            impl Drop for MyType {
                fn drop(&mut self) {
                    self.cleanup();
                }
            }
        };

        let mut detector = PatternDetector::new();
        let matches = detector.analyze_file(&code);

        assert!(matches.iter().any(|m| m.pattern == "resource_management"));
    }
}
