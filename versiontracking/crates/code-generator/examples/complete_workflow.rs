use anyhow::Result;
use ast_analyzer::Analyzer;
use code_generator::{
    CodeGenerator, GenerationTarget, GeneratorConfig, Template,
    transform::{AsyncTransform, ErrorHandlingTransform, SafetyTransform, TransformConfig},
    patterns::{BuilderPattern, AsyncWrapperPattern},
};
use quote::quote;
use syn::parse_quote;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up configuration
    let config = GeneratorConfig {
        transform_config: TransformConfig {
            async_conversion: true,
            error_handling: true,
            safety_checks: true,
            ..Default::default()
        },
        documentation: true,
        ..Default::default()
    };

    // Create generator with transforms
    let mut generator = CodeGenerator::new(config.clone());
    generator.register_transform(Box::new(AsyncTransform { config: config.transform_config.clone() }));
    generator.register_transform(Box::new(ErrorHandlingTransform { config: config.transform_config.clone() }));
    generator.register_transform(Box::new(SafetyTransform { config: config.transform_config.clone() }));

    // Register templates
    generator.register_template("custom_struct", Template::new("custom_struct")
        .with_generator(|ctx| {
            let name = ctx.get_string("struct_name")?;
            let fields = ctx.get_list("fields")?;
            
            let struct_name = syn::Ident::new(&name, proc_macro2::Span::call_site());
            let field_tokens = fields.iter().map(|f| {
                quote! { pub #f: String }
            });

            Ok(quote! {
                #[derive(Debug, Clone)]
                pub struct #struct_name {
                    #(#field_tokens,)*
                }
            })
        }));

    // Sample source code to analyze
    let source: syn::File = parse_quote! {
        use std::fs;

        pub struct Config {
            path: String,
            options: Vec<String>,
        }

        impl Config {
            pub fn load(path: &str) -> Result<Self, std::io::Error> {
                let content = fs::read_to_string(path)?;
                Ok(Self {
                    path: path.to_string(),
                    options: vec![],
                })
            }
        }
    };

    // Analyze source
    let analyzer = Analyzer::new();
    let analysis = analyzer.analyze(&source);

    println!("Analysis found {} patterns", analysis.patterns.len());

    // Generate new code
    let generation_target = GenerationTarget::Struct {
        name: "GeneratedConfig".to_string(),
        fields: vec!["path".to_string(), "options".to_string()],
    };

    let code = generator.generate_from_source(&source, generation_target).await?;
    println!("Generated code:\n{}", code);

    // Apply builder pattern
    let builder_pattern = BuilderPattern::new(config.transform_config.clone());
    let mut template_ctx = crate::template::TemplateContext::new();
    template_ctx.set("struct_name", "GeneratedConfig");
    template_ctx.set("fields", vec!["path", "options"]);

    let builder_code = builder_pattern.generate(&template_ctx)?;
    println!("\nBuilder pattern code:\n{}", builder_code);

    // Apply async wrapper
    let async_pattern = AsyncWrapperPattern::new(config.transform_config);
    template_ctx.set("function_name", "load");
    
    let async_code = async_pattern.generate(&template_ctx)?;
    println!("\nAsync wrapper code:\n{}", async_code);

    Ok(())
}

// Example struct using generated code
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_code_generation() -> Result<()> {
        let mut generator = CodeGenerator::new(GeneratorConfig::default());
        
        let target = GenerationTarget::Struct {
            name: "TestConfig".to_string(),
            fields: vec!["field1".to_string(), "field2".to_string()],
        };

        let code = generator.generate(target).await?;
        let code_str = code.to_string();
        
        assert!(code_str.contains("TestConfig"));
        assert!(code_str.contains("field1"));
        assert!(code_str.contains("field2"));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_pattern_application() -> Result<()> {
        let config = TransformConfig::default();
        let pattern = BuilderPattern::new(config);
        
        let mut ctx = crate::template::TemplateContext::new();
        ctx.set("struct_name", "TestStruct");
        ctx.set("fields", vec!["field1", "field2"]);
        
        let code = pattern.generate(&ctx)?;
        let code_str = code.to_string();
        
        assert!(code_str.contains("TestStructBuilder"));
        assert!(code_str.contains("build"));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_transformation_chain() -> Result<()> {
        let config = GeneratorConfig::default();
        let mut generator = CodeGenerator::new(config.clone());
        
        generator.register_transform(Box::new(AsyncTransform {
            config: config.transform_config.clone(),
        }));
        
        let target = GenerationTarget::Function {
            name: "test_fn".to_string(),
            is_async: false,
            is_unsafe: false,
        };

        let code = generator.generate(target).await?;
        let code_str = code.to_string();
        
        assert!(code_str.contains("test_fn"));
        
        Ok(())
    }
}