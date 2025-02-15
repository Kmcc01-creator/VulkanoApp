use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_quote, visit_mut::VisitMut, Error, Expr, ExprMatch, ExprTry, Item, ItemFn, Pat, Result,
    ReturnType, Type,
};

/// Describes how to handle errors in transformed code
#[derive(Debug, Clone)]
pub struct ErrorHandlingConfig {
    /// Whether to use anyhow for error handling
    pub use_anyhow: bool,
    /// Whether to generate custom error types
    pub generate_custom_errors: bool,
    /// Custom error type name (if generating)
    pub custom_error_name: Option<String>,
    /// Whether to add context to error sites
    pub add_error_context: bool,
}

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        Self {
            use_anyhow: true,
            generate_custom_errors: false,
            custom_error_name: None,
            add_error_context: true,
        }
    }
}

pub struct ErrorTransformer {
    config: ErrorHandlingConfig,
    error_sites: Vec<ErrorSite>,
}

#[derive(Debug)]
struct ErrorSite {
    span: proc_macro2::Span,
    error_type: String,
    context: Option<String>,
}

impl ErrorTransformer {
    pub fn new(config: ErrorHandlingConfig) -> Self {
        Self {
            config,
            error_sites: Vec::new(),
        }
    }

    pub fn transform_item(&mut self, item: &mut Item) -> Result<()> {
        match item {
            Item::Fn(f) => self.transform_fn(f),
            _ => Ok(()),
        }
    }

    fn transform_fn(&mut self, func: &mut ItemFn) -> Result<()> {
        // Only transform functions that return Result
        if let ReturnType::Type(_, ty) = &mut func.sig.output {
            if self.is_result_type(ty) {
                // Transform the return type
                self.transform_return_type(ty)?;

                // Transform the function body
                let mut visitor = ErrorVisitor::new(&self.config);
                visitor.visit_block_mut(&mut func.block);
                self.error_sites.extend(visitor.error_sites);
            }
        }
        Ok(())
    }

    fn transform_return_type(&self, ty: &mut Type) -> Result<()> {
        if self.config.use_anyhow {
            // Replace with anyhow::Result<T>
            *ty = parse_quote!(anyhow::Result<_>);
        } else if self.config.generate_custom_errors {
            if let Some(error_name) = &self.config.custom_error_name {
                // Replace with custom error type
                *ty = parse_quote!(Result<_, #error_name>);
            }
        }
        Ok(())
    }

    fn is_result_type(&self, ty: &Type) -> bool {
        if let Type::Path(type_path) = ty {
            type_path
                .path
                .segments
                .last()
                .map_or(false, |s| s.ident == "Result")
        } else {
            false
        }
    }
}

struct ErrorVisitor {
    config: ErrorHandlingConfig,
    error_sites: Vec<ErrorSite>,
}

impl ErrorVisitor {
    fn new(config: &ErrorHandlingConfig) -> Self {
        Self {
            config: config.clone(),
            error_sites: Vec::new(),
        }
    }

    fn add_error_context(&self, expr: &Expr) -> TokenStream {
        if !self.config.add_error_context {
            return expr.to_token_stream();
        }

        let context = format!("Error at {}", expr.span().start().line);
        quote! {
            #expr.context(#context)
        }
    }
}

impl VisitMut for ErrorVisitor {
    fn visit_expr_try_mut(&mut self, expr: &mut ExprTry) {
        // Transform ? operator usage
        if self.config.use_anyhow {
            let inner = &expr.expr;
            let context = self.add_error_context(&*expr.expr);
            *expr = parse_quote! {
                #inner.context("Operation failed")?
            };
        }
        syn::visit_mut::visit_expr_try_mut(self, expr);
    }

    fn visit_expr_match_mut(&mut self, expr: &mut ExprMatch) {
        // Transform error matching patterns
        if self.config.use_anyhow {
            if let Expr::Path(ref path) = &*expr.expr {
                if path
                    .path
                    .segments
                    .last()
                    .map_or(false, |s| s.ident == "Err")
                {
                    // Transform error match arms to use anyhow
                    for arm in &mut expr.arms {
                        if let Pat::TupleStruct(ref mut pat) = arm.pat {
                            if pat.path.segments.last().map_or(false, |s| s.ident == "Err") {
                                *pat = parse_quote!(Err(e));
                            }
                        }
                    }
                }
            }
        }
        syn::visit_mut::visit_expr_match_mut(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_error_transformation() {
        let mut func: ItemFn = parse_quote! {
            fn process_data() -> Result<String, std::io::Error> {
                let data = std::fs::read_to_string("test.txt")?;
                Ok(data)
            }
        };

        let config = ErrorHandlingConfig {
            use_anyhow: true,
            add_error_context: true,
            ..Default::default()
        };

        let mut transformer = ErrorTransformer::new(config);
        transformer.transform_fn(&mut func).unwrap();

        let result = quote!(#func).to_string();
        assert!(result.contains("anyhow::Result"));
        assert!(result.contains("context"));
    }

    #[test]
    fn test_custom_error_type() {
        let mut func: ItemFn = parse_quote! {
            fn process_data() -> Result<String, std::io::Error> {
                let data = std::fs::read_to_string("test.txt")?;
                Ok(data)
            }
        };

        let config = ErrorHandlingConfig {
            use_anyhow: false,
            generate_custom_errors: true,
            custom_error_name: Some("AppError".to_string()),
            add_error_context: false,
        };

        let mut transformer = ErrorTransformer::new(config);
        transformer.transform_fn(&mut func).unwrap();

        let result = quote!(#func).to_string();
        assert!(result.contains("Result<_, AppError>"));
    }
}
