use proc_macro2::TokenStream;
use std::collections::HashMap;
use syn::{
    Attribute, File, FnArg, GenericParam, Generics, Ident, Item, ItemFn, ItemImpl, ItemStruct,
    ItemTrait, Path, PathArguments, Signature, Type, TypePath, Visibility,
};

#[derive(Debug)]
pub struct ExtractedType {
    pub name: String,
    pub generics: Vec<String>,
    pub traits: Vec<String>,
    pub methods: Vec<ExtractedMethod>,
    pub fields: Vec<ExtractedField>,
    pub visibility: String,
    pub attributes: Vec<String>,
}

#[derive(Debug)]
pub struct ExtractedMethod {
    pub name: String,
    pub signature: Signature,
    pub visibility: String,
    pub is_async: bool,
    pub is_unsafe: bool,
    pub attributes: Vec<String>,
}

#[derive(Debug)]
pub struct ExtractedField {
    pub name: String,
    pub ty: Type,
    pub visibility: String,
    pub attributes: Vec<String>,
}

pub(crate) fn extract_signatures(file: &File) -> HashMap<String, Signature> {
    let mut signatures = HashMap::new();

    for item in &file.items {
        match item {
            Item::Fn(f) => {
                signatures.insert(f.sig.ident.to_string(), f.sig.clone());
            }
            Item::Impl(i) => {
                for item in &i.items {
                    if let syn::ImplItem::Fn(method) = item {
                        let key =
                            format!("{}::{}", type_path_to_string(&i.self_ty), method.sig.ident);
                        signatures.insert(key, method.sig.clone());
                    }
                }
            }
            Item::Trait(t) => {
                for item in &t.items {
                    if let syn::TraitItem::Fn(method) = item {
                        let key = format!("{}::{}", t.ident, method.sig.ident);
                        signatures.insert(key, method.sig.clone());
                    }
                }
            }
            _ => {}
        }
    }

    signatures
}

pub fn extract_type_info(item: &Item) -> Option<ExtractedType> {
    match item {
        Item::Struct(s) => Some(extract_struct_info(s)),
        Item::Impl(i) => Some(extract_impl_info(i)),
        Item::Trait(t) => Some(extract_trait_info(t)),
        _ => None,
    }
}

fn extract_struct_info(strukt: &ItemStruct) -> ExtractedType {
    ExtractedType {
        name: strukt.ident.to_string(),
        generics: extract_generics(&strukt.generics),
        traits: Vec::new(),  // Traits are found in impl blocks
        methods: Vec::new(), // Methods are found in impl blocks
        fields: strukt
            .fields
            .iter()
            .map(|f| ExtractedField {
                name: f.ident.as_ref().map_or("".to_string(), |i| i.to_string()),
                ty: f.ty.clone(),
                visibility: visibility_to_string(&f.vis),
                attributes: extract_attributes(&f.attrs),
            })
            .collect(),
        visibility: visibility_to_string(&strukt.vis),
        attributes: extract_attributes(&strukt.attrs),
    }
}

fn extract_impl_info(impl_: &ItemImpl) -> ExtractedType {
    ExtractedType {
        name: type_path_to_string(&impl_.self_ty),
        generics: extract_generics(&impl_.generics),
        traits: impl_
            .trait_
            .as_ref()
            .map(|(_, path, _)| path_to_string(path))
            .into_iter()
            .collect(),
        methods: impl_
            .items
            .iter()
            .filter_map(|item| {
                if let syn::ImplItem::Fn(method) = item {
                    Some(ExtractedMethod {
                        name: method.sig.ident.to_string(),
                        signature: method.sig.clone(),
                        visibility: visibility_to_string(&method.vis),
                        is_async: method.sig.asyncness.is_some(),
                        is_unsafe: method.sig.unsafety.is_some(),
                        attributes: extract_attributes(&method.attrs),
                    })
                } else {
                    None
                }
            })
            .collect(),
        fields: Vec::new(),
        visibility: "".to_string(), // Impls don't have visibility
        attributes: extract_attributes(&impl_.attrs),
    }
}

fn extract_trait_info(trait_: &ItemTrait) -> ExtractedType {
    ExtractedType {
        name: trait_.ident.to_string(),
        generics: extract_generics(&trait_.generics),
        traits: trait_
            .supertraits
            .iter()
            .map(|bound| bound.to_token_stream().to_string())
            .collect(),
        methods: trait_
            .items
            .iter()
            .filter_map(|item| {
                if let syn::TraitItem::Fn(method) = item {
                    Some(ExtractedMethod {
                        name: method.sig.ident.to_string(),
                        signature: method.sig.clone(),
                        visibility: "pub".to_string(), // Trait items are always public
                        is_async: method.sig.asyncness.is_some(),
                        is_unsafe: method.sig.unsafety.is_some(),
                        attributes: extract_attributes(&method.attrs),
                    })
                } else {
                    None
                }
            })
            .collect(),
        fields: Vec::new(),
        visibility: visibility_to_string(&trait_.vis),
        attributes: extract_attributes(&trait_.attrs),
    }
}

fn extract_generics(generics: &Generics) -> Vec<String> {
    generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(ty) => Some(ty.ident.to_string()),
            GenericParam::Lifetime(lt) => Some(lt.lifetime.ident.to_string()),
            GenericParam::Const(c) => Some(c.ident.to_string()),
        })
        .collect()
}

fn extract_attributes(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .map(|attr| attr.to_token_stream().to_string())
        .collect()
}

fn visibility_to_string(vis: &Visibility) -> String {
    match vis {
        Visibility::Public(_) => "pub".to_string(),
        Visibility::Restricted(vis) => format!("pub({})", vis.to_token_stream()),
        Visibility::Inherited => "".to_string(),
    }
}

fn type_path_to_string(ty: &Type) -> String {
    if let Type::Path(TypePath { path, .. }) = ty {
        path_to_string(path)
    } else {
        ty.to_token_stream().to_string()
    }
}

fn path_to_string(path: &Path) -> String {
    path.segments
        .iter()
        .map(|seg| {
            let mut s = seg.ident.to_string();
            if let PathArguments::AngleBracketed(args) = &seg.arguments {
                s.push('<');
                s.push_str(&args.to_token_stream().to_string());
                s.push('>');
            }
            s
        })
        .collect::<Vec<_>>()
        .join("::")
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_extract_struct_info() {
        let code: ItemStruct = parse_quote! {
            #[derive(Debug)]
            pub struct Test<T> {
                pub field: T,
                internal: String,
            }
        };

        let info = extract_struct_info(&code);
        assert_eq!(info.name, "Test");
        assert_eq!(info.generics, vec!["T"]);
        assert_eq!(info.fields.len(), 2);
        assert_eq!(info.visibility, "pub");
        assert!(!info.attributes.is_empty());
    }

    #[test]
    fn test_extract_trait_info() {
        let code: ItemTrait = parse_quote! {
            pub trait TestTrait: Debug {
                fn required_method(&self);
                fn optional_method(&self) {
                    // Default impl
                }
            }
        };

        let info = extract_trait_info(&code);
        assert_eq!(info.name, "TestTrait");
        assert!(!info.traits.is_empty()); // Has Debug supertrait
        assert_eq!(info.methods.len(), 2);
    }

    #[test]
    fn test_extract_impl_info() {
        let code: ItemImpl = parse_quote! {
            impl<T: Debug> Test<T> {
                pub fn new() -> Self {
                    Self { field: None }
                }

                pub async fn async_method(&self) -> Result<(), Error> {
                    Ok(())
                }
            }
        };

        let info = extract_impl_info(&code);
        assert_eq!(info.name, "Test < T >");
        assert_eq!(info.methods.len(), 2);
        assert!(info.methods[1].is_async);
    }
}
