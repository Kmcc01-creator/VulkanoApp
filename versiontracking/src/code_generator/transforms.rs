use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::{
    parse_quote, spanned::Spanned, visit_mut::VisitMut, Block, Error, Expr, File, FnArg, Ident,
    Item, ItemFn, ReturnType, Type,
};

use super::patterns::{Pattern, PatternAnalyzer, PatternType};

// Core types and traits first
#[derive(Debug, Clone)]
pub enum TransformationType {
    AsyncifyFunction {
        add_runtime: bool,
        preserve_sync_version: bool,
    },
    OptimizeErrorHandling {
        use_anyhow: bool,
        generate_custom_errors: bool,
    },
    SimplifyLifetimes {
        elide_when_possible: bool,
        suggest_static: bool,
    },
    ExtractInterface {
        public: bool,
        include_docs: bool,
    },
    OptimizeConcurrency {
        use_async: bool,
        add_sync_guards: bool,
    },
}

pub trait TransformationValidator {
    fn validate(&self, result: &TokenStream) -> Result<(), Error>;
    fn validate_safety(&self, result: &TokenStream) -> Result<(), Error>;
    fn validate_performance(
        &self,
        original: &TokenStream,
        result: &TokenStream,
    ) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct BlockingOperation {
    pub name: String,
    pub span: Span,
    pub async_alternative: Option<String>,
    pub runtime_required: bool,
}

// Implementation of BlockingOperation
impl BlockingOperation {
    fn new(name: &str, span: Span) -> Self {
        let async_alternative = match name {
            "std::fs::read_to_string" => Some("tokio::fs::read_to_string".to_string()),
            "std::fs::write" => Some("tokio::fs::write".to_string()),
            "std::fs::read" => Some("tokio::fs::read".to_string()),
            "std::thread::sleep" => Some("tokio::time::sleep".to_string()),
            "std::net::TcpStream::connect" => Some("tokio::net::TcpStream::connect".to_string()),
            _ => None,
        };
        let runtime_required = async_alternative.is_some();
        Self {
            name: name.to_string(),
            span,
            async_alternative,
            runtime_required,
        }
    }
}

// Validator implementation
pub struct AsyncTransformValidator {
    required_features: Vec<&'static str>,
}

impl AsyncTransformValidator {
    pub fn new() -> Self {
        Self {
            required_features: vec!["tokio/fs", "tokio/time", "tokio/net"],
        }
    }
}

impl TransformationValidator for AsyncTransformValidator {
    fn validate(&self, _result: &TokenStream) -> Result<(), Error> {
        // TODO: Implement proper validation
        Ok(())
    }

    fn validate_safety(&self, _result: &TokenStream) -> Result<(), Error> {
        Ok(())
    }

    fn validate_performance(
        &self,
        _original: &TokenStream,
        _result: &TokenStream,
    ) -> Result<(), Error> {
        Ok(())
    }
}

// Analysis components
struct BlockingCallAnalyzer {
    blocking_ops: Vec<BlockingOperation>,
}

impl BlockingCallAnalyzer {
    fn new() -> Self {
        Self {
            blocking_ops: Vec::new(),
        }
    }

    fn analyze_block(&mut self, block: &Block) {
        use syn::visit::Visit;

        struct Visitor<'a> {
            ops: &'a mut Vec<BlockingOperation>,
        }

        impl<'ast> Visit<'ast> for Visitor<'_> {
            fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
                if let syn::Expr::Path(path) = &*call.func {
                    let name = path
                        .path
                        .segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");

                    self.ops
                        .push(BlockingOperation::new(&name, call.func.span()));
                }
                syn::visit::visit_expr_call(self, call);
            }

            fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
                let method_name = call.method.to_string();
                if method_name.starts_with("blocking_") {
                    self.ops
                        .push(BlockingOperation::new(&method_name, call.method.span()));
                }
                syn::visit::visit_expr_method_call(self, call);
            }
        }

        let mut visitor = Visitor {
            ops: &mut self.blocking_ops,
        };
        visitor.visit_block(block);
    }
}

// Transformation components
struct AsyncifyVisitor {
    add_runtime: bool,
    preserve_sync: bool,
    analyzer: BlockingCallAnalyzer,
}

impl AsyncifyVisitor {
    fn new(add_runtime: bool, preserve_sync: bool) -> Self {
        Self {
            add_runtime,
            preserve_sync,
            analyzer: BlockingCallAnalyzer::new(),
        }
    }

    fn asyncify_function(&self, func: &mut ItemFn) -> Option<ItemFn> {
        if func.sig.asyncness.is_some() {
            return None;
        }

        let mut analyzer = BlockingCallAnalyzer::new();
        analyzer.analyze_block(&func.block);

        if analyzer.blocking_ops.is_empty() {
            return None;
        }

        let mut async_fn = func.clone();
        async_fn.sig.asyncness = Some(parse_quote!(async));

        if self.add_runtime {
            self.wrap_blocking_calls(&mut async_fn.block, &analyzer.blocking_ops);
        }

        Some(async_fn)
    }

    fn wrap_blocking_calls(&self, block: &mut Block, ops: &[BlockingOperation]) {
        let mut transformer = AsyncCallTransformer::new(ops);
        transformer.visit_block_mut(block);
    }
}

impl VisitMut for AsyncifyVisitor {
    fn visit_item_fn_mut(&mut self, func: &mut ItemFn) {
        if let Some(async_fn) = self.asyncify_function(func) {
            if self.preserve_sync {
                let sync_name = format!("{}_sync", func.sig.ident);
                func.sig.ident = Ident::new(&sync_name, func.sig.ident.span());
            } else {
                *func = async_fn;
            }
        }
        syn::visit_mut::visit_item_fn_mut(self, func);
    }
}

struct AsyncCallTransformer {
    ops: Vec<BlockingOperation>,
}

impl AsyncCallTransformer {
    fn new(ops: &[BlockingOperation]) -> Self {
        Self { ops: ops.to_vec() }
    }
}

impl VisitMut for AsyncCallTransformer {
    fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
        if let syn::Expr::Path(path) = &*call.func {
            let name = path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if let Some(op) = self.ops.iter().find(|op| op.name == name) {
                if let Some(ref async_alt) = op.async_alternative {
                    let new_path: syn::Path = syn::parse_str(async_alt).unwrap();
                    call.func = Box::new(Expr::Path(syn::ExprPath {
                        attrs: vec![],
                        qself: None,
                        path: new_path,
                    }));
                }
            }
        }
        syn::visit_mut::visit_expr_call_mut(self, call);
    }
}

// Main transformer
pub struct CodeTransformer {
    transformations: Vec<TransformationType>,
    validators: Vec<Box<dyn TransformationValidator>>,
    pattern_analyzer: PatternAnalyzer,
}

impl Default for CodeTransformer {
    fn default() -> Self {
        Self {
            transformations: Vec::new(),
            validators: vec![Box::new(AsyncTransformValidator::new())],
            pattern_analyzer: PatternAnalyzer::new(),
        }
    }
}

impl CodeTransformer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_transformation(&mut self, transformation: TransformationType) {
        self.transformations.push(transformation);
    }

    pub fn transform_ast(&mut self, ast: &File) -> Result<TokenStream, Error> {
        let mut transformed_ast = ast.clone();
        let patterns = self.pattern_analyzer.analyze_file(&transformed_ast);
        let mut needs_runtime = false;

        for transformation in &self.transformations {
            match transformation {
                TransformationType::AsyncifyFunction {
                    add_runtime,
                    preserve_sync_version,
                } => {
                    if *add_runtime {
                        needs_runtime = true;
                    }
                    let mut visitor = AsyncifyVisitor::new(*add_runtime, *preserve_sync_version);
                    visitor.visit_file_mut(&mut transformed_ast);
                }
                TransformationType::OptimizeErrorHandling {
                    use_anyhow,
                    generate_custom_errors,
                } => {
                    let config = super::error_handling::ErrorHandlingConfig {
                        use_anyhow: *use_anyhow,
                        generate_custom_errors: *generate_custom_errors,
                        custom_error_name: None,
                        add_error_context: true,
                    };
                    let mut error_transformer =
                        super::error_handling::ErrorTransformer::new(config);
                    for item in &mut transformed_ast.items {
                        error_transformer.transform_item(item)?;
                    }
                }
                _ => {}
            }
        }

        let result = quote! { #transformed_ast };

        for validator in &self.validators {
            validator.validate(&result)?;
            validator.validate_safety(&result)?;
        }

        Ok(result)
    }

    pub fn get_patterns(&mut self) -> Vec<Pattern> {
        let empty_file = syn::parse_str("").unwrap();
        self.pattern_analyzer.analyze_file(&empty_file)
    }

    pub fn analyze_file(&mut self, ast: &File) -> Vec<Pattern> {
        self.pattern_analyzer.analyze_file(ast)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn test_asyncify_blocking_function() {
        let code = r#"
            fn blocking_function() -> Result<String, std::io::Error> {
                std::fs::read_to_string("test.txt")
            }
        "#;

        let mut transformer = CodeTransformer::new();
        transformer.add_transformation(TransformationType::AsyncifyFunction {
            add_runtime: true,
            preserve_sync_version: false,
        });

        let ast = parse_str::<File>(code).unwrap();
        let result = transformer.transform_ast(&ast).unwrap();

        assert!(result.to_string().contains("async"));
        assert!(result.to_string().contains("tokio::fs::read_to_string"));
    }

    #[test]
    fn test_pattern_detection() {
        let code = r#"
            fn blocking_with_unsafe() {
                unsafe {
                    std::fs::read_to_string("test.txt").unwrap();
                }
            }
        "#;

        let mut transformer = CodeTransformer::new();
        let ast = parse_str::<File>(code).unwrap();
        let patterns = transformer.analyze_file(&ast);

        assert!(patterns
            .iter()
            .any(|p| matches!(p.pattern_type, PatternType::BlockingCall)));
        assert!(patterns
            .iter()
            .any(|p| matches!(p.pattern_type, PatternType::UnsafeUsage)));
    }
}
