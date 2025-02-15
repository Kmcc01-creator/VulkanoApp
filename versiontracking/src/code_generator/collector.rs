use super::types::*;
use quote::ToTokens;
use std::collections::HashMap;
use syn::{
    punctuated::Punctuated, visit::Visit, Attribute, File, Ident, ImplItem, Item, ItemFn, ItemImpl,
    ItemStruct, ItemTrait, Path, Token, TraitItem, Type,
};

#[derive(Default)]
pub struct ModuleCollector {
    module_structure: ModuleStructure,
    current_scope: String,
}

impl ModuleCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_module_structure(self) -> ModuleStructure {
        self.module_structure
    }

    fn is_doc_attribute(attr: &&Attribute) -> bool {
        attr.path
            .segments
            .first()
            .map(|seg| seg.ident == "doc")
            .unwrap_or(false)
    }

    fn is_derive_attribute(attr: &&Attribute) -> bool {
        attr.path
            .segments
            .first()
            .map(|seg| seg.ident == "derive")
            .unwrap_or(false)
    }

    fn extract_documentation(attrs: &[Attribute]) -> Option<String> {
        attrs
            .iter()
            .filter(Self::is_doc_attribute)
            .filter_map(|attr| {
                let tokens = attr.to_token_stream().to_string();
                // Extract the doc string from #[doc = "..."]
                let start = tokens.find('"')?;
                let end = tokens.rfind('"')?;
                Some(tokens[start + 1..end].to_string())
            })
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
            .into()
    }

    fn analyze_type_dependencies(&mut self, ty: &Type) {
        // TODO: Implement recursive type analysis
        // This will populate self.module_structure.type_dependencies
    }
}

impl<'ast> Visit<'ast> for ModuleCollector {
    fn visit_item_fn(&mut self, func: &'ast ItemFn) {
        self.module_structure.functions.push(FunctionInfo {
            name: func.sig.ident.to_string(),
            signature: func.sig.to_token_stream().to_string(),
            dependencies: Vec::new(),
            is_public: matches!(func.vis, syn::Visibility::Public(_)),
            documentation: Self::extract_documentation(&func.attrs),
        });
        syn::visit::visit_item_fn(self, func);
    }

    fn visit_item_struct(&mut self, item_struct: &'ast ItemStruct) {
        let fields = item_struct
            .fields
            .iter()
            .filter_map(|field| {
                Some(FieldInfo {
                    name: field.ident.as_ref()?.to_string(),
                    type_name: field.ty.to_token_stream().to_string(),
                    is_public: matches!(field.vis, syn::Visibility::Public(_)),
                    documentation: Self::extract_documentation(&field.attrs),
                })
            })
            .collect();

        let derives = item_struct
            .attrs
            .iter()
            .filter(Self::is_derive_attribute)
            .flat_map(|attr| {
                attr.to_token_stream()
                    .to_string()
                    .trim_start_matches("derive")
                    .trim_matches(|c| c == '(' || c == ')')
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>()
            })
            .collect();

        self.module_structure.structs.push(StructInfo {
            name: item_struct.ident.to_string(),
            fields,
            derives,
            is_public: matches!(item_struct.vis, syn::Visibility::Public(_)),
            documentation: Self::extract_documentation(&item_struct.attrs),
        });

        syn::visit::visit_item_struct(self, item_struct);
    }

    fn visit_item_trait(&mut self, item_trait: &'ast ItemTrait) {
        let mut methods = Vec::new();
        for item in &item_trait.items {
            if let TraitItem::Method(method) = item {
                methods.push(TraitMethodInfo {
                    name: method.sig.ident.to_string(),
                    signature: method.sig.to_token_stream().to_string(),
                    has_default_impl: method.default.is_some(),
                    documentation: Self::extract_documentation(&method.attrs),
                });
            }
        }

        self.module_structure.traits.push(TraitInfo {
            name: item_trait.ident.to_string(),
            methods,
            supertraits: item_trait
                .supertraits
                .iter()
                .map(|t| t.to_token_stream().to_string())
                .collect(),
            is_public: matches!(item_trait.vis, syn::Visibility::Public(_)),
            documentation: Self::extract_documentation(&item_trait.attrs),
        });
        syn::visit::visit_item_trait(self, item_trait);
    }

    fn visit_item_impl(&mut self, item_impl: &'ast ItemImpl) {
        let mut methods = Vec::new();
        for item in &item_impl.items {
            if let ImplItem::Method(method) = item {
                methods.push(FunctionInfo {
                    name: method.sig.ident.to_string(),
                    signature: method.sig.to_token_stream().to_string(),
                    dependencies: Vec::new(),
                    is_public: matches!(method.vis, syn::Visibility::Public(_)),
                    documentation: Self::extract_documentation(&method.attrs),
                });
            }
        }

        self.module_structure.implementations.push(ImplInfo {
            target_type: item_impl.self_ty.to_token_stream().to_string(),
            trait_name: item_impl
                .trait_
                .as_ref()
                .map(|(_, path, _)| path.to_token_stream().to_string()),
            methods,
        });
        syn::visit::visit_item_impl(self, item_impl);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn test_collect_functions() {
        let code = r#"
            pub fn function1() {}
            fn function2() {}
        "#;

        let syntax = syn::parse_str::<File>(code).unwrap();
        let mut collector = ModuleCollector::new();
        syn::visit::visit_file(&mut collector, &syntax);
        let structure = collector.get_module_structure();

        assert_eq!(structure.functions.len(), 2);
        assert!(structure
            .functions
            .iter()
            .any(|f| f.name == "function1" && f.is_public));
        assert!(structure
            .functions
            .iter()
            .any(|f| f.name == "function2" && !f.is_public));
    }

    #[test]
    fn test_collect_structs() {
        let code = r#"
            #[derive(Debug, Clone)]
            pub struct TestStruct {
                pub field1: String,
                field2: i32,
            }
        "#;

        let syntax = syn::parse_str::<File>(code).unwrap();
        let mut collector = ModuleCollector::new();
        syn::visit::visit_file(&mut collector, &syntax);
        let structure = collector.get_module_structure();

        assert_eq!(structure.structs.len(), 1);
        let test_struct = &structure.structs[0];
        assert_eq!(test_struct.name, "TestStruct");
        assert!(test_struct.is_public);
        assert_eq!(test_struct.fields.len(), 2);
        assert!(test_struct.derives.contains(&"Debug".to_string()));
        assert!(test_struct.derives.contains(&"Clone".to_string()));
    }
}
