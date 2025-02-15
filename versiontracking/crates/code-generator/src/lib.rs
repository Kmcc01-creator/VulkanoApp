use std::collections::HashMap;
use anyhow::Result;
use ast_analyzer::{Analyzer, AnalysisResult, PatternMatch};
use proc_macro2::TokenStream;
use quote::quote;
use syn::File;

mod generator;
mod transform;
mod template;
mod patterns;

pub use generator::{CodeGenerator, GeneratorConfig};
pub use transform::{Transform, TransformConfig};
pub use template::{Template, TemplateContext};

#[derive(Debug)]
pub enum GenerationTarget {
    Function {
        name: String,
        is_async: bool,
        is_unsafe: bool,
    },
    Struct {
        name: String,
        fields: Vec<String>,
    },
    Trait {
        name: String,
        methods: Vec<String>,
    },
    Module {
        name: String,
        items: Vec<GenerationTarget>,
    },
}

#[derive(Debug)]
pub struct GenerationContext {
    pub target: GenerationTarget,
    pub patterns: Vec<PatternMatch>,
    pub dependencies: HashMap<String, String>,
    pub config: GeneratorConfig,
}

pub struct Generator {
    analyzer: Analyzer,
    transforms: Vec<Box<dyn Transform>>,
    templates: HashMap<String, Template>,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            analyzer: Analyzer::new(),
            transforms: Vec::new(),
            templates: HashMap::new(),
        }
    }

    pub fn register_transform(&mut self, transform: Box<dyn Transform>) {
        self.transforms.push(transform);
    }

    pub fn register_template(&mut self, name: String, template: Template) {
        self.templates.insert(name, template);
    }

    pub async fn generate(&self, source: &File, target: GenerationTarget) -> Result<TokenStream> {
        // Analyze source code
        let analysis = self.analyzer.analyze(source);
        
        // Create generation context
        let context = self.create_context(target, &analysis)?;
        
        // Apply transforms
        let mut code = self.generate_initial_code(&context)?;
        for transform in &self.transforms {
            code = transform.apply(code, &context).await?;
        }

        Ok(code)
    }

    fn create_context(&self, target: GenerationTarget, analysis: &AnalysisResult) -> Result<GenerationContext> {
        Ok(GenerationContext {
            target,
            patterns: analysis.patterns.clone(),
            dependencies: HashMap::new(), // Will be populated based on analysis
            config: GeneratorConfig::default(),
        })
    }

    fn generate_initial_code(&self, context: &GenerationContext) -> Result<TokenStream> {
        match &context.target {
            GenerationTarget::Function { name, is_async, is_unsafe } => {
                let fn_name = syn::Ident::new(name, proc_macro2::Span::call_site());
                let async_token = if *is_async {
                    quote!(async)
                } else {
                    quote!()
                };
                let unsafe_token = if *is_unsafe {
                    quote!(unsafe)
                } else {
                    quote!()
                };
                
                Ok(quote! {
                    #async_token #unsafe_token fn #fn_name() {
                        // Generated function body
                    }
                })
            }
            GenerationTarget::Struct { name, fields } => {
                let struct_name = syn::Ident::new(name, proc_macro2::Span::call_site());
                let fields = fields.iter().map(|f| {
                    let field_name = syn::Ident::new(f, proc_macro2::Span::call_site());
                    quote! { pub #field_name: () }
                });
                
                Ok(quote! {
                    pub struct #struct_name {
                        #(#fields,)*
                    }
                })
            }
            GenerationTarget::Trait { name, methods } => {
                let trait_name = syn::Ident::new(name, proc_macro2::Span::call_site());
                let methods = methods.iter().map(|m| {
                    let method_name = syn::Ident::new(m, proc_macro2::Span::call_site());
                    quote! { fn #method_name(&self); }
                });
                
                Ok(quote! {
                    pub trait #trait_name {
                        #(#methods)*
                    }
                })
            }
            GenerationTarget::Module { name, items } => {
                let mod_name = syn::Ident::new(name, proc_macro2::Span::call_site());
                let mut content = TokenStream::new();
                
                for item in items {
                    let ctx = GenerationContext {
                        target: item.clone(),
                        patterns: context.patterns.clone(),
                        dependencies: context.dependencies.clone(),
                        config: context.config.clone(),
                    };
                    content.extend(self.generate_initial_code(&ctx)?);
                }
                
                Ok(quote! {
                    pub mod #mod_name {
                        #content
                    }
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[tokio::test]
    async fn test_function_generation() {
        let generator = Generator::new();
        let source: File = parse_quote! {
            fn test() {}
        };

        let target = GenerationTarget::Function {
            name: "generated_fn".to_string(),
            is_async: true,
            is_unsafe: false,
        };

        let result = generator.generate(&source, target).await.unwrap();
        let code = result.to_string();
        assert!(code.contains("async fn generated_fn"));
    }

    #[tokio::test]
    async fn test_struct_generation() {
        let generator = Generator::new();
        let source: File = parse_quote! {
            struct Test {}
        };

        let target = GenerationTarget::Struct {
            name: "Generated".to_string(),
            fields: vec!["field1".to_string(), "field2".to_string()],
        };

        let result = generator.generate(&source, target).await.unwrap();
        let code = result.to_string();
        assert!(code.contains("struct Generated"));
        assert!(code.contains("field1"));
        assert!(code.contains("field2"));
    }
}