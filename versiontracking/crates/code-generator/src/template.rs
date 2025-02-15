use anyhow::{Context, Result};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::collections::HashMap;
use syn::{Ident, Type};

#[derive(Debug, Clone)]
pub struct TemplateContext {
    variables: HashMap<String, TemplateValue>,
    config: TemplateConfig,
}

#[derive(Debug, Clone)]
pub enum TemplateValue {
    String(String),
    Number(i64),
    Boolean(bool),
    Type(Type),
    List(Vec<TemplateValue>),
    Map(HashMap<String, TemplateValue>),
}

#[derive(Debug, Clone)]
pub struct TemplateConfig {
    pub indent_style: IndentStyle,
    pub doc_style: DocStyle,
    pub visibility: VisibilityStyle,
}

#[derive(Debug, Clone, Copy)]
pub enum IndentStyle {
    Spaces(u8),
    Tabs,
}

#[derive(Debug, Clone, Copy)]
pub enum DocStyle {
    None,
    Basic,
    Detailed,
}

#[derive(Debug, Clone, Copy)]
pub enum VisibilityStyle {
    AllPublic,
    MixedVisibility,
    ModulePrivate,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            indent_style: IndentStyle::Spaces(4),
            doc_style: DocStyle::Basic,
            visibility: VisibilityStyle::MixedVisibility,
        }
    }
}

#[derive(Debug)]
pub struct Template {
    name: String,
    content: TemplateContent,
    parameters: Vec<TemplateParameter>,
}

#[derive(Debug)]
enum TemplateContent {
    Raw(String),
    TokenStream(TokenStream),
    Function(Box<dyn Fn(&TemplateContext) -> Result<TokenStream> + Send + Sync>),
}

#[derive(Debug)]
struct TemplateParameter {
    name: String,
    type_name: String,
    required: bool,
    default_value: Option<TemplateValue>,
}

impl Template {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content: TemplateContent::Raw(String::new()),
            parameters: Vec::new(),
        }
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = TemplateContent::Raw(content.into());
        self
    }

    pub fn with_tokens(mut self, tokens: TokenStream) -> Self {
        self.content = TemplateContent::TokenStream(tokens);
        self
    }

    pub fn with_generator(
        mut self,
        generator: impl Fn(&TemplateContext) -> Result<TokenStream> + Send + Sync + 'static,
    ) -> Self {
        self.content = TemplateContent::Function(Box::new(generator));
        self
    }

    pub fn add_parameter(
        mut self,
        name: impl Into<String>,
        type_name: impl Into<String>,
        required: bool,
        default_value: Option<TemplateValue>,
    ) -> Self {
        self.parameters.push(TemplateParameter {
            name: name.into(),
            type_name: type_name.into(),
            required,
            default_value,
        });
        self
    }

    pub fn render(&self, context: &TemplateContext) -> Result<TokenStream> {
        // Validate parameters
        self.validate_parameters(context)?;

        // Generate code based on content type
        match &self.content {
            TemplateContent::Raw(content) => {
                let parsed: TokenStream = content
                    .parse()
                    .context("Failed to parse template content as TokenStream")?;
                Ok(parsed)
            }
            TemplateContent::TokenStream(tokens) => Ok(tokens.clone()),
            TemplateContent::Function(generator) => generator(context),
        }
    }

    fn validate_parameters(&self, context: &TemplateContext) -> Result<()> {
        for param in &self.parameters {
            if param.required && !context.variables.contains_key(&param.name) {
                anyhow::bail!("Required parameter '{}' not provided", param.name);
            }
        }
        Ok(())
    }
}

/// Common template implementations
pub mod templates {
    use super::*;

    pub fn struct_template() -> Template {
        Template::new("struct")
            .add_parameter("name", "String", true, None)
            .add_parameter("fields", "Vec<(String, String)>", true, None)
            .with_generator(|ctx| {
                let name = ctx.get_string("name")?;
                let fields = ctx.get_list("fields")?;

                let struct_name = Ident::new(&name, proc_macro2::Span::call_site());
                let field_tokens = fields
                    .iter()
                    .map(|field| {
                        let (name, ty) = field.as_tuple()?;
                        let field_name = Ident::new(&name, proc_macro2::Span::call_site());
                        let field_type: Type = syn::parse_str(&ty)?;
                        Ok(quote! { pub #field_name: #field_type })
                    })
                    .collect::<Result<Vec<_>>>()?;

                Ok(quote! {
                    pub struct #struct_name {
                        #(#field_tokens,)*
                    }
                })
            })
    }

    pub fn trait_template() -> Template {
        Template::new("trait")
            .add_parameter("name", "String", true, None)
            .add_parameter("methods", "Vec<String>", true, None)
            .with_generator(|ctx| {
                let name = ctx.get_string("name")?;
                let methods = ctx.get_list("methods")?;

                let trait_name = Ident::new(&name, proc_macro2::Span::call_site());
                let method_tokens = methods.iter().map(|method| {
                    let method_name = Ident::new(method, proc_macro2::Span::call_site());
                    quote! { fn #method_name(&self); }
                });

                Ok(quote! {
                    pub trait #trait_name {
                        #(#method_tokens)*
                    }
                })
            })
    }
}

impl TemplateContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            config: TemplateConfig::default(),
        }
    }

    pub fn with_config(config: TemplateConfig) -> Self {
        Self {
            variables: HashMap::new(),
            config,
        }
    }

    pub fn set<T: Into<TemplateValue>>(&mut self, name: impl Into<String>, value: T) {
        self.variables.insert(name.into(), value.into());
    }

    pub fn get(&self, name: &str) -> Option<&TemplateValue> {
        self.variables.get(name)
    }

    pub fn get_string(&self, name: &str) -> Result<String> {
        match self.get(name) {
            Some(TemplateValue::String(s)) => Ok(s.clone()),
            _ => anyhow::bail!("Variable '{}' not found or not a string", name),
        }
    }

    pub fn get_list(&self, name: &str) -> Result<Vec<TemplateValue>> {
        match self.get(name) {
            Some(TemplateValue::List(l)) => Ok(l.clone()),
            _ => anyhow::bail!("Variable '{}' not found or not a list", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_struct_template() {
        let mut context = TemplateContext::new();
        context.set("name", "TestStruct".to_string());
        context.set(
            "fields",
            vec![
                TemplateValue::List(vec![
                    TemplateValue::String("field1".to_string()),
                    TemplateValue::String("String".to_string()),
                ]),
                TemplateValue::List(vec![
                    TemplateValue::String("field2".to_string()),
                    TemplateValue::String("i32".to_string()),
                ]),
            ],
        );

        let template = templates::struct_template();
        let result = template.render(&context).unwrap();

        assert!(result.to_string().contains("TestStruct"));
        assert!(result.to_string().contains("field1"));
        assert!(result.to_string().contains("field2"));
    }

    #[test]
    fn test_trait_template() {
        let mut context = TemplateContext::new();
        context.set("name", "TestTrait".to_string());
        context.set(
            "methods",
            vec![
                TemplateValue::String("method1".to_string()),
                TemplateValue::String("method2".to_string()),
            ],
        );

        let template = templates::trait_template();
        let result = template.render(&context).unwrap();

        assert!(result.to_string().contains("TestTrait"));
        assert!(result.to_string().contains("method1"));
        assert!(result.to_string().contains("method2"));
    }
}
