use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

use crate::template::TemplateContext;
use crate::transform::TransformConfig;

/// Common code generation patterns that can be applied to different targets
pub struct GenerationPattern {
    name: String,
    description: String,
    applicability: PatternApplicability,
    generator: Box<dyn PatternGenerator>,
}

#[derive(Debug, Clone, Copy)]
pub enum PatternApplicability {
    Functions,
    Structs,
    Traits,
    All,
}

/// Trait for implementing code generation patterns
pub trait PatternGenerator: Send + Sync {
    fn generate(&self, context: &TemplateContext) -> Result<TokenStream>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn applicability(&self) -> PatternApplicability;
}

/// Builder getter/setter methods for structs
pub struct BuilderPattern {
    config: TransformConfig,
}

impl BuilderPattern {
    pub fn new(config: TransformConfig) -> Self {
        Self { config }
    }
}

impl PatternGenerator for BuilderPattern {
    fn name(&self) -> &str {
        "builder_pattern"
    }

    fn description(&self) -> &str {
        "Generates builder pattern implementation for structs"
    }

    fn applicability(&self) -> PatternApplicability {
        PatternApplicability::Structs
    }

    fn generate(&self, context: &TemplateContext) -> Result<TokenStream> {
        let struct_name = context.get_string("struct_name")?;
        let fields = context.get_list("fields")?;

        let struct_ident = Ident::new(&struct_name, proc_macro2::Span::call_site());
        let builder_ident = Ident::new(
            &format!("{}Builder", struct_name),
            proc_macro2::Span::call_site(),
        );

        let field_defs = fields.iter().map(|field| {
            let name = Ident::new(field, proc_macro2::Span::call_site());
            quote! { #name: Option<String> }
        });

        let build_setters = fields.iter().map(|field| {
            let name = Ident::new(field, proc_macro2::Span::call_site());
            quote! {
                pub fn #name(mut self, value: impl Into<String>) -> Self {
                    self.#name = Some(value.into());
                    self
                }
            }
        });

        Ok(quote! {
            pub struct #builder_ident {
                #(#field_defs,)*
            }

            impl #builder_ident {
                pub fn new() -> Self {
                    Self {
                        #(#fields: None,)*
                    }
                }

                #(#build_setters)*

                pub fn build(self) -> Result<#struct_ident, &'static str> {
                    Ok(#struct_ident {
                        #(#fields: self.#fields.ok_or("Missing field")?,)*
                    })
                }
            }

            impl #struct_ident {
                pub fn builder() -> #builder_ident {
                    #builder_ident::new()
                }
            }
        })
    }
}

/// Async wrapper pattern for functions
pub struct AsyncWrapperPattern {
    config: TransformConfig,
}

impl AsyncWrapperPattern {
    pub fn new(config: TransformConfig) -> Self {
        Self { config }
    }
}

impl PatternGenerator for AsyncWrapperPattern {
    fn name(&self) -> &str {
        "async_wrapper"
    }

    fn description(&self) -> &str {
        "Wraps synchronous functions with async versions"
    }

    fn applicability(&self) -> PatternApplicability {
        PatternApplicability::Functions
    }

    fn generate(&self, context: &TemplateContext) -> Result<TokenStream> {
        let fn_name = context.get_string("function_name")?;
        let async_fn_name = format!("{}_async", fn_name);

        let fn_ident = Ident::new(&fn_name, proc_macro2::Span::call_site());
        let async_fn_ident = Ident::new(&async_fn_name, proc_macro2::Span::call_site());

        Ok(quote! {
            pub async fn #async_fn_ident() -> Result<()> {
                tokio::task::spawn_blocking(move || {
                    #fn_ident()
                }).await?
            }
        })
    }
}

/// Default trait implementation pattern
pub struct DefaultImplPattern {
    config: TransformConfig,
}

impl DefaultImplPattern {
    pub fn new(config: TransformConfig) -> Self {
        Self { config }
    }
}

impl PatternGenerator for DefaultImplPattern {
    fn name(&self) -> &str {
        "default_impl"
    }

    fn description(&self) -> &str {
        "Generates default trait implementations"
    }

    fn applicability(&self) -> PatternApplicability {
        PatternApplicability::Traits
    }

    fn generate(&self, context: &TemplateContext) -> Result<TokenStream> {
        let trait_name = context.get_string("trait_name")?;
        let methods = context.get_list("methods")?;

        let trait_ident = Ident::new(&trait_name, proc_macro2::Span::call_site());
        let default_ident = Ident::new(
            &format!("Default{}", trait_name),
            proc_macro2::Span::call_site(),
        );

        let method_impls = methods.iter().map(|method| {
            let method_ident = Ident::new(method, proc_macro2::Span::call_site());
            quote! {
                fn #method_ident(&self) {
                    // Default implementation
                }
            }
        });

        Ok(quote! {
            pub struct #default_ident;

            impl #trait_ident for #default_ident {
                #(#method_impls)*
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_pattern() {
        let config = TransformConfig::default();
        let pattern = BuilderPattern::new(config);

        let mut context = TemplateContext::new();
        context.set("struct_name", "TestStruct");
        context.set("fields", vec!["field1", "field2"]);

        let result = pattern.generate(&context).unwrap();
        let code = result.to_string();

        assert!(code.contains("TestStructBuilder"));
        assert!(code.contains("field1"));
        assert!(code.contains("field2"));
        assert!(code.contains("build"));
    }

    #[test]
    fn test_async_wrapper() {
        let config = TransformConfig::default();
        let pattern = AsyncWrapperPattern::new(config);

        let mut context = TemplateContext::new();
        context.set("function_name", "test_function");

        let result = pattern.generate(&context).unwrap();
        let code = result.to_string();

        assert!(code.contains("test_function_async"));
        assert!(code.contains("spawn_blocking"));
    }
}
