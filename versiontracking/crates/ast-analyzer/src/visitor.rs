use proc_macro2::Span;
use std::collections::HashSet;
use syn::{
    visit::{self, Visit},
    Block, ExprUnsafe, File, ImplItem, ItemFn, ItemImpl, ItemStruct, ItemTrait, TraitItem,
};

use crate::{AnalysisMetrics, Pattern, PatternKind};

pub(crate) struct AstVisitor {
    pub(crate) metrics: AnalysisMetrics,
    patterns: Vec<Pattern>,
    unsafe_blocks: HashSet<Span>,
    blocking_calls: HashSet<Span>,
}

impl AstVisitor {
    pub(crate) fn new() -> Self {
        Self {
            metrics: AnalysisMetrics::default(),
            patterns: Vec::new(),
            unsafe_blocks: HashSet::new(),
            blocking_calls: HashSet::new(),
        }
    }

    fn analyze_blocking_call(&mut self, span: Span, path: &str) {
        if !self.blocking_calls.contains(&span) {
            self.blocking_calls.insert(span);
            self.metrics.blocking_call_count += 1;

            self.patterns.push(Pattern {
                kind: PatternKind::BlockingCall,
                span,
                context: format!("Blocking call detected: {}", path),
            });
        }
    }

    fn analyze_unsafe_block(&mut self, span: Span) {
        if !self.unsafe_blocks.contains(&span) {
            self.unsafe_blocks.insert(span);
            self.metrics.unsafe_block_count += 1;

            self.patterns.push(Pattern {
                kind: PatternKind::UnsafeBlock,
                span,
                context: "Unsafe block detected".to_string(),
            });
        }
    }

    fn calculate_complexity(&self, block: &Block) -> f64 {
        let mut complexity = 1.0;

        struct ComplexityVisitor {
            score: f64,
        }

        impl<'ast> Visit<'ast> for ComplexityVisitor {
            fn visit_expr_if(&mut self, _: &'ast syn::ExprIf) {
                self.score += 1.0;
                // Continue visiting nested expressions
            }

            fn visit_expr_match(&mut self, expr: &'ast syn::ExprMatch) {
                self.score += expr.arms.len() as f64 * 0.5;
                // Continue visiting nested expressions
            }

            fn visit_expr_loop(&mut self, _: &'ast syn::ExprLoop) {
                self.score += 2.0;
                // Continue visiting nested expressions
            }

            fn visit_expr_while(&mut self, _: &'ast syn::ExprWhile) {
                self.score += 2.0;
                // Continue visiting nested expressions
            }

            fn visit_expr_for_loop(&mut self, _: &'ast syn::ExprForLoop) {
                self.score += 1.5;
                // Continue visiting nested expressions
            }

            fn visit_expr_try(&mut self, _: &'ast syn::ExprTry) {
                self.score += 0.5;
                // Continue visiting nested expressions
            }
        }

        let mut visitor = ComplexityVisitor { score: complexity };
        visitor.visit_block(block);
        visitor.score
    }
}

impl<'ast> Visit<'ast> for AstVisitor {
    fn visit_file(&mut self, file: &'ast File) {
        visit::visit_file(self, file);
    }

    fn visit_item_fn(&mut self, func: &'ast ItemFn) {
        self.metrics.function_count += 1;

        // Calculate complexity
        let complexity = self.calculate_complexity(&func.block);
        self.metrics.complexity_score += complexity;

        if complexity > 10.0 {
            self.patterns.push(Pattern {
                kind: PatternKind::PerformanceCritical,
                span: func.sig.ident.span(),
                context: format!("High complexity function (score: {})", complexity),
            });
        }

        visit::visit_item_fn(self, func);
    }

    fn visit_item_struct(&mut self, strukt: &'ast ItemStruct) {
        self.metrics.struct_count += 1;

        // Check for resource types that might need cleanup
        let has_resource_fields = strukt.fields.iter().any(|field| {
            let type_name = format!("{:?}", field.ty);
            type_name.contains("File")
                || type_name.contains("Socket")
                || type_name.contains("Connection")
                || type_name.contains("Mutex")
                || type_name.contains("Lock")
        });

        if has_resource_fields {
            self.patterns.push(Pattern {
                kind: PatternKind::ResourceLeak,
                span: strukt.ident.span(),
                context: format!("Struct '{}' contains resource types", strukt.ident),
            });
        }

        visit::visit_item_struct(self, strukt);
    }

    fn visit_item_trait(&mut self, trait_: &'ast ItemTrait) {
        self.metrics.trait_count += 1;

        for item in &trait_.items {
            if let TraitItem::Fn(method) = item {
                let is_unsafe = method.sig.unsafety.is_some();
                if is_unsafe {
                    self.analyze_unsafe_block(method.sig.ident.span());
                }
            }
        }

        visit::visit_item_trait(self, trait_);
    }

    fn visit_item_impl(&mut self, impl_: &'ast ItemImpl) {
        for item in &impl_.items {
            if let ImplItem::Fn(method) = item {
                let is_unsafe = method.sig.unsafety.is_some();
                if is_unsafe {
                    self.analyze_unsafe_block(method.sig.ident.span());
                }
            }
        }

        visit::visit_item_impl(self, impl_);
    }

    fn visit_expr_unsafe(&mut self, expr: &'ast ExprUnsafe) {
        self.analyze_unsafe_block(expr.unsafe_token.span);
        visit::visit_expr_unsafe(self, expr);
    }

    fn visit_expr_call(&mut self, expr: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*expr.func {
            let path_string = path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            // Detect blocking calls
            if path_string.contains("fs::")
                || path_string.contains("thread::sleep")
                || path_string.contains("stdin")
                || path_string.contains("stdout")
            {
                self.analyze_blocking_call(expr.func.span(), &path_string);
            }

            // Detect security-sensitive operations
            if path_string.contains("unsafe_")
                || path_string.contains("raw_")
                || path_string.contains("transmute")
            {
                self.patterns.push(Pattern {
                    kind: PatternKind::SecuritySensitive,
                    span: expr.func.span(),
                    context: format!("Security-sensitive operation: {}", path_string),
                });
            }
        }

        visit::visit_expr_call(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_detect_unsafe_blocks() {
        let code: File = parse_quote! {
            fn test_function() {
                unsafe {
                    let ptr = std::ptr::null();
                }
            }
        };

        let mut visitor = AstVisitor::new();
        visitor.visit_file(&code);

        assert!(visitor.metrics.unsafe_block_count > 0);
        assert!(visitor
            .patterns
            .iter()
            .any(|p| matches!(p.kind, PatternKind::UnsafeBlock)));
    }

    #[test]
    fn test_detect_blocking_calls() {
        let code: File = parse_quote! {
            fn test_function() {
                std::fs::read_to_string("test.txt").unwrap();
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        };

        let mut visitor = AstVisitor::new();
        visitor.visit_file(&code);

        assert!(visitor.metrics.blocking_call_count > 0);
        assert!(visitor
            .patterns
            .iter()
            .any(|p| matches!(p.kind, PatternKind::BlockingCall)));
    }

    #[test]
    fn test_complexity_calculation() {
        let code: File = parse_quote! {
            fn complex_function() {
                for i in 0..10 {
                    if i % 2 == 0 {
                        match i {
                            0 => println!("zero"),
                            _ => println!("non-zero"),
                        }
                    } else {
                        while i > 5 {
                            println!("high");
                        }
                    }
                }
            }
        };

        let mut visitor = AstVisitor::new();
        visitor.visit_file(&code);

        assert!(visitor.metrics.complexity_score > 5.0);
    }
}
