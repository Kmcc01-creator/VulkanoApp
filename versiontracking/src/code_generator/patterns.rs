use proc_macro2::TokenStream;
use syn::{File, Item};

#[derive(Debug, Default)]
pub struct PatternAnalyzer {
    detected_patterns: Vec<Pattern>,
}

#[derive(Debug)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub location: proc_macro2::Span,
    pub description: String,
}

#[derive(Debug)]
pub enum PatternType {
    BlockingCall,
    ErrorPropagation,
    ResourceLeak,
    Synchronization,
    UnsafeUsage,
    PerformanceCritical,
}

impl PatternAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn analyze_file(&mut self, file: &File) -> Vec<Pattern> {
        self.detected_patterns.clear();
        self.visit_file(file);
        self.detected_patterns.clone()
    }

    fn visit_file(&mut self, file: &File) {
        for item in &file.items {
            self.analyze_item(item);
        }
    }

    fn analyze_item(&mut self, item: &Item) {
        match item {
            Item::Fn(func) => self.analyze_function(func),
            Item::Impl(impl_block) => self.analyze_impl(impl_block),
            Item::Struct(struct_def) => self.analyze_struct(struct_def),
            _ => {}
        }
    }

    fn analyze_function(&mut self, func: &syn::ItemFn) {
        // Analyze for blocking calls
        if self.contains_blocking_calls(&func.block) {
            self.detected_patterns.push(Pattern {
                pattern_type: PatternType::BlockingCall,
                location: func.sig.ident.span(),
                description: format!("Function '{}' contains blocking calls", func.sig.ident),
            });
        }

        // Analyze for unsafe usage
        if func
            .block
            .stmts
            .iter()
            .any(|stmt| self.contains_unsafe(stmt))
        {
            self.detected_patterns.push(Pattern {
                pattern_type: PatternType::UnsafeUsage,
                location: func.sig.ident.span(),
                description: format!("Function '{}' contains unsafe code", func.sig.ident),
            });
        }
    }

    fn analyze_impl(&mut self, impl_block: &syn::ItemImpl) {
        for item in &impl_block.items {
            if let syn::ImplItem::Method(method) = item {
                self.analyze_function(&syn::ItemFn {
                    attrs: method.attrs.clone(),
                    vis: method.vis.clone(),
                    sig: method.sig.clone(),
                    block: method.block.clone(),
                });
            }
        }
    }

    fn analyze_struct(&mut self, struct_def: &syn::ItemStruct) {
        // Check for resource types that might need cleanup
        for field in &struct_def.fields {
            if self.is_resource_type(&field.ty) {
                self.detected_patterns.push(Pattern {
                    pattern_type: PatternType::ResourceLeak,
                    location: field.ident.as_ref().unwrap().span(),
                    description: format!(
                        "Field '{}' might need explicit cleanup",
                        field.ident.as_ref().unwrap()
                    ),
                });
            }
        }
    }

    fn contains_blocking_calls(&self, block: &syn::Block) -> bool {
        use syn::visit::Visit;

        struct BlockingCallVisitor {
            found_blocking: bool,
        }

        impl<'ast> Visit<'ast> for BlockingCallVisitor {
            fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
                if let syn::Expr::Path(path) = &*call.func {
                    let name = path
                        .path
                        .segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");

                    if name.contains("fs::") || name.contains("thread::") {
                        self.found_blocking = true;
                    }
                }
                syn::visit::visit_expr_call(self, call);
            }
        }

        let mut visitor = BlockingCallVisitor {
            found_blocking: false,
        };
        visitor.visit_block(block);
        visitor.found_blocking
    }

    fn contains_unsafe(&self, stmt: &syn::Stmt) -> bool {
        match stmt {
            syn::Stmt::Expr(syn::Expr::Unsafe(_), _) => true,
            syn::Stmt::Block(block) => block.stmts.iter().any(|s| self.contains_unsafe(s)),
            _ => false,
        }
    }

    fn is_resource_type(&self, ty: &syn::Type) -> bool {
        if let syn::Type::Path(path) = ty {
            let type_name = path.path.segments.last().unwrap().ident.to_string();
            matches!(
                type_name.as_str(),
                "File" | "TcpStream" | "MutexGuard" | "Connection" | "Transaction"
            )
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_detect_blocking_call() {
        let code: File = parse_quote! {
            fn blocking_function() {
                std::fs::read_to_string("test.txt");
            }
        };

        let mut analyzer = PatternAnalyzer::new();
        let patterns = analyzer.analyze_file(&code);

        assert!(!patterns.is_empty());
        assert!(matches!(
            patterns[0].pattern_type,
            PatternType::BlockingCall
        ));
    }

    #[test]
    fn test_detect_unsafe_usage() {
        let code: File = parse_quote! {
            fn unsafe_function() {
                unsafe {
                    let ptr = std::ptr::null();
                }
            }
        };

        let mut analyzer = PatternAnalyzer::new();
        let patterns = analyzer.analyze_file(&code);

        assert!(!patterns.is_empty());
        assert!(matches!(patterns[0].pattern_type, PatternType::UnsafeUsage));
    }
}
