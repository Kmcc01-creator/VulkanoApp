use std::collections::HashMap;
use anyhow::{Context, Result};
use proc_macro2::TokenStream;
use quote::quote;
use syn::File;

use crate::{
    Template, TemplateContext, Transform, TransformConfig,
    GenerationTarget, GenerationContext, TemplateValue,
};

#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub transform_config: TransformConfig,
    pub template_context: TemplateContext,
    pub custom_templates: bool,
    pub documentation: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            transform_config: TransformConfig::default(),
            template_context: TemplateContext::new(),
            custom_templates: false,
            documentation: true,
        }
    }
}

pub struct CodeGenerator {
    config: GeneratorConfig,
    templates: HashMap<String, Template>,
    transforms: Vec<Box<dyn Transform>>,
}

impl CodeGenerator {
    pub fn new(config: GeneratorConfig) -> Self {
        Self {
            config,
            templates: HashMap::new(),
            transforms: Vec::new(),
        }
    }

    pub fn register_template(&mut self, name: impl Into<String>, template: Template) {
        self.templates.insert(name.into(), template);
    }

    pub fn register_transform(&mut self, transform: Box<dyn Transform>) {
        self.transforms.push(transform);
    }

    pub async fn generate(&self, target: GenerationTarget) -> Result<TokenStream> {
        let context = self.create_context(&target)?;
        
        // Generate initial code from template
        let template_name = self.get_template_name(&target);
        let template = self.templates.get(&template_name)
            .with_context(|| format!("Template not found: {}", template_name))?;
        
        let mut code = template.render(&context)?;

        // Apply transforms
        for transform in &self.transforms {
            code = transform.apply(code, &context).await?;
        }

        // Add documentation if enabled
        if self.config.documentation {
            code = self.add_documentation(code, &context)?;
        }

        Ok(code)
    }

    pub async fn generate_from_source(&self, source: &File, target: GenerationTarget) -> Result<TokenStream> {
        // Analyze source file first
        let ast_analyzer = ast_analyzer::Analyzer::new();
        let analysis = ast_analyzer.analyze(source);

        // Create enhanced context with analysis results
        let mut context = self.create_context(&target)?;
        context.patterns = analysis.patterns;

        // Generate code with analysis information
        let template_name = self.get_template_name(&target);
        let template = self.templates.get(&template_name)
            .with_context(|| format!("Template not found: {}", template_name))?;
        
        let mut code = template.render(&context)?;

        // Apply transforms with analysis information
        for transform in &self.transforms {
            code = transform.apply(code, &context).await?;
        }

        Ok(code)
    }

    fn create_context(&self, target: &GenerationTarget) -> Result<GenerationContext> {
        let mut context = GenerationContext {
            target: target.clone(),
            patterns: Vec::new(),
            dependencies: HashMap::new(),
            config: self.config.clone(),
        };

        // Add target-specific template variables
        match target {
            GenerationTarget::Function { name, is_async, is_unsafe } => {
                let mut template_ctx = self.config.template_context.clone();
                template_ctx.set("function_name", name.clone());
                template_ctx.set("is_async", *is_async);
                template_ctx.set("is_unsafe", *is_unsafe);
            }
            GenerationTarget::Struct { name, fields } => {
                let mut template_ctx = self.config.template_context.clone();
                template_ctx.set("struct_name", name.clone());
                template_ctx.set("fields", TemplateValue::List(
                    fields.iter()
                        .map(|f| TemplateValue::String(f.clone()))
                        .collect()
                ));
            }
            GenerationTarget::Trait { name, methods } => {
                let mut template_ctx = self.config.template_context.clone();
                template_ctx.set("trait_name", name.clone());
                template_ctx.set("methods", TemplateValue::List(
                    methods.iter()
                        .map(|m| TemplateValue::String(m.clone()))
                        .collect()
                ));
            }
            GenerationTarget::Module { name, items } => {
                let mut template_ctx = self.config.template_context.clone();
                template_ctx.set("module_name", name.clone());
            }
        }

        Ok(context)
    }

    fn get_template_name(&self, target: &GenerationTarget) -> String {
        match target {
            GenerationTarget::Function { .. } => "function".to_string(),
            GenerationTarget::Struct { .. } => "struct".to_string(),
            GenerationTarget::Trait { .. } => "trait".to_string(),
            GenerationTarget::Module { .. } => "module".to_string(),
        }
    }

    fn add_documentation(&self, code: TokenStream, context: &GenerationContext) -> Result<TokenStream> {
        let doc = match &context.target {
            GenerationTarget::Function { name, .. } => {
                format!(" Auto-generated function `{}`", name)
            }
            GenerationTarget::Struct { name, .. } => {
                format!(" Auto-generated struct `{}`", name)
            }
            GenerationTarget::Trait { name, .. } => {
                format!(" Auto-generated trait `{}`", name)
            }
            GenerationTarget::Module { name, .. } => {
                format!(" Auto-generated module `{}`", name)
            }
        };

        Ok(quote! {
            #[doc = #doc]
            #code
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::templates;

    #[tokio::test]
    async fn test_function_generation() {
        let mut generator = CodeGenerator::new(GeneratorConfig::default());
        generator.register_template("function", Template::new("function")
            .with_generator(|ctx| {
                let name = ctx.get_string("function_name")?;
                let fn_name = syn::Ident::new(&name, proc_macro2::Span::call_site());
                Ok(quote! {
                    fn #fn_name() {}
                })
            }));

        let target = GenerationTarget::Function {
            name: "test_fn".to_string(),
            is_async: false,
            is_unsafe: false,
        };

        let result = generator.generate(target).await.unwrap();
        assert!(result.to_string().contains("test_fn"));
    }

    #[tokio::test]
    async fn test_struct_generation() {
        let mut generator = CodeGenerator::new(GeneratorConfig::default());
        generator.register_template("struct", templates::struct_template());

        let target = GenerationTarget::Struct {
            name: "TestStruct".to_string(),
            fields: vec!["field1: String".to_string(), "field2: i32".to_string()],
        };

        let result = generator.generate(target).await.unwrap();
        let code = result.to_string();
        assert!(code.contains("TestStruct"));
        assert!(code.contains("field1"));
        assert!(code.contains("field2"));
    }
}