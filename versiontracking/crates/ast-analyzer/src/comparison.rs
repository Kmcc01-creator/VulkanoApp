use proc_macro2::Span;
use std::collections::HashMap;
use syn::{
    File, Item, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait, Signature, Type, TypePath,
    Visibility,
};

use crate::BreakingChange;
use crate::ChangeKind;

pub(crate) fn compare_files(old_file: &File, new_file: &File) -> Vec<BreakingChange> {
    let mut changes = Vec::new();

    // First, build maps of items by name for efficient comparison
    let old_items = collect_items_by_name(old_file);
    let new_items = collect_items_by_name(new_file);

    // Check for removed items (public APIs)
    for (name, old_item) in &old_items {
        if is_public(old_item) && !new_items.contains_key(name) {
            changes.push(BreakingChange {
                kind: ChangeKind::MethodRemoved {
                    method_name: name.clone(),
                    type_name: "".to_string(),
                },
                span: get_item_span(old_item),
                description: format!("Public item removed: {}", name),
                suggested_fix: None,
            });
        }
    }

    // Compare remaining items for breaking changes
    for (name, new_item) in &new_items {
        if let Some(old_item) = old_items.get(name) {
            changes.extend(compare_items(old_item, new_item));
        }
    }

    changes
}

fn collect_items_by_name(file: &File) -> HashMap<String, &Item> {
    let mut items = HashMap::new();

    for item in &file.items {
        if let Some(name) = get_item_name(item) {
            items.insert(name, item);
        }
    }

    items
}

fn get_item_name(item: &Item) -> Option<String> {
    match item {
        Item::Fn(f) => Some(f.sig.ident.to_string()),
        Item::Struct(s) => Some(s.ident.to_string()),
        Item::Trait(t) => Some(t.ident.to_string()),
        Item::Impl(i) => i
            .self_ty
            .clone()
            .to_token_stream()
            .to_string()
            .replace(' ', ""),
        Item::Mod(m) => Some(m.ident.to_string()),
        _ => None,
    }
}

fn is_public(item: &Item) -> bool {
    match item {
        Item::Fn(f) => matches!(f.vis, Visibility::Public(_)),
        Item::Struct(s) => matches!(s.vis, Visibility::Public(_)),
        Item::Trait(t) => matches!(t.vis, Visibility::Public(_)),
        Item::Mod(m) => matches!(m.vis, Visibility::Public(_)),
        _ => false,
    }
}

fn get_item_span(item: &Item) -> Span {
    match item {
        Item::Fn(f) => f.sig.ident.span(),
        Item::Struct(s) => s.ident.span(),
        Item::Trait(t) => t.ident.span(),
        Item::Impl(i) => i.self_ty.span(),
        Item::Mod(m) => m.ident.span(),
        _ => Span::call_site(),
    }
}

fn compare_items(old_item: &Item, new_item: &Item) -> Vec<BreakingChange> {
    let mut changes = Vec::new();

    match (old_item, new_item) {
        (Item::Fn(old_fn), Item::Fn(new_fn)) => {
            changes.extend(compare_functions(old_fn, new_fn));
        }
        (Item::Struct(old_struct), Item::Struct(new_struct)) => {
            changes.extend(compare_structs(old_struct, new_struct));
        }
        (Item::Trait(old_trait), Item::Trait(new_trait)) => {
            changes.extend(compare_traits(old_trait, new_trait));
        }
        (Item::Impl(old_impl), Item::Impl(new_impl)) => {
            changes.extend(compare_impls(old_impl, new_impl));
        }
        _ => {}
    }

    changes
}

fn compare_functions(old_fn: &ItemFn, new_fn: &ItemFn) -> Vec<BreakingChange> {
    let mut changes = Vec::new();

    // Check visibility reduction
    if let (Visibility::Public(_), _) = (&old_fn.vis, &new_fn.vis) {
        if !matches!(new_fn.vis, Visibility::Public(_)) {
            changes.push(BreakingChange {
                kind: ChangeKind::VisibilityReduced {
                    item_name: old_fn.sig.ident.to_string(),
                    old_vis: "pub".to_string(),
                    new_vis: format!("{:?}", new_fn.vis),
                },
                span: old_fn.sig.ident.span(),
                description: format!("Function visibility reduced: {}", old_fn.sig.ident),
                suggested_fix: None,
            });
        }
    }

    // Compare signatures
    if compare_signatures(&old_fn.sig, &new_fn.sig) {
        changes.push(BreakingChange {
            kind: ChangeKind::SignatureChanged {
                old_sig: format!("{:?}", old_fn.sig),
                new_sig: format!("{:?}", new_fn.sig),
            },
            span: old_fn.sig.ident.span(),
            description: format!("Function signature changed: {}", old_fn.sig.ident),
            suggested_fix: None,
        });
    }

    changes
}

fn compare_signatures(old_sig: &Signature, new_sig: &Signature) -> bool {
    // Return true if signatures are incompatible
    if old_sig.inputs.len() != new_sig.inputs.len() {
        return true;
    }

    let inputs_changed = old_sig
        .inputs
        .iter()
        .zip(new_sig.inputs.iter())
        .any(|(old, new)| {
            !types_are_compatible(&extract_type_from_fnarg(old), &extract_type_from_fnarg(new))
        });

    let return_changed = match (&old_sig.output, &new_sig.output) {
        (syn::ReturnType::Default, syn::ReturnType::Default) => false,
        (syn::ReturnType::Type(_, old_ty), syn::ReturnType::Type(_, new_ty)) => {
            !types_are_compatible(old_ty, new_ty)
        }
        _ => true,
    };

    inputs_changed || return_changed
}

fn extract_type_from_fnarg(arg: &syn::FnArg) -> &Type {
    match arg {
        syn::FnArg::Typed(pat_type) => &*pat_type.ty,
        syn::FnArg::Receiver(_) => panic!("self parameter not handled"),
    }
}

fn types_are_compatible(old_ty: &Type, new_ty: &Type) -> bool {
    match (old_ty, new_ty) {
        (
            Type::Path(TypePath { path: old_path, .. }),
            Type::Path(TypePath { path: new_path, .. }),
        ) => old_path
            .segments
            .iter()
            .zip(new_path.segments.iter())
            .all(|(old_seg, new_seg)| old_seg.ident == new_seg.ident),
        _ => false,
    }
}

fn compare_structs(old_struct: &ItemStruct, new_struct: &ItemStruct) -> Vec<BreakingChange> {
    let mut changes = Vec::new();

    let old_fields: HashMap<_, _> = old_struct
        .fields
        .iter()
        .filter_map(|f| Some((f.ident.as_ref()?.to_string(), f)))
        .collect();

    // Check for removed public fields
    for (name, field) in &old_fields {
        if matches!(field.vis, Visibility::Public(_)) {
            if !new_struct
                .fields
                .iter()
                .any(|f| f.ident.as_ref().map_or(false, |i| i.to_string() == *name))
            {
                changes.push(BreakingChange {
                    kind: ChangeKind::MethodRemoved {
                        method_name: name.clone(),
                        type_name: old_struct.ident.to_string(),
                    },
                    span: field.ident.as_ref().unwrap().span(),
                    description: format!("Public field removed: {}", name),
                    suggested_fix: None,
                });
            }
        }
    }

    changes
}

fn compare_traits(old_trait: &ItemTrait, new_trait: &ItemTrait) -> Vec<BreakingChange> {
    let mut changes = Vec::new();

    // Check for removed trait methods
    for old_item in &old_trait.items {
        if let syn::TraitItem::Fn(old_method) = old_item {
            let method_exists = new_trait.items.iter().any(|new_item| {
                if let syn::TraitItem::Fn(new_method) = new_item {
                    old_method.sig.ident == new_method.sig.ident
                } else {
                    false
                }
            });

            if !method_exists {
                changes.push(BreakingChange {
                    kind: ChangeKind::MethodRemoved {
                        method_name: old_method.sig.ident.to_string(),
                        type_name: old_trait.ident.to_string(),
                    },
                    span: old_method.sig.ident.span(),
                    description: format!(
                        "Trait method removed: {}::{}",
                        old_trait.ident, old_method.sig.ident
                    ),
                    suggested_fix: None,
                });
            }
        }
    }

    changes
}

fn compare_impls(old_impl: &ItemImpl, new_impl: &ItemImpl) -> Vec<BreakingChange> {
    let mut changes = Vec::new();

    // Compare method implementations
    for old_item in &old_impl.items {
        if let syn::ImplItem::Fn(old_method) = old_item {
            let method_exists = new_impl.items.iter().any(|new_item| {
                if let syn::ImplItem::Fn(new_method) = new_item {
                    old_method.sig.ident == new_method.sig.ident
                } else {
                    false
                }
            });

            if !method_exists && matches!(old_method.vis, Visibility::Public(_)) {
                changes.push(BreakingChange {
                    kind: ChangeKind::MethodRemoved {
                        method_name: old_method.sig.ident.to_string(),
                        type_name: old_impl.self_ty.to_token_stream().to_string(),
                    },
                    span: old_method.sig.ident.span(),
                    description: format!(
                        "Implementation method removed: {}::{}",
                        old_impl.self_ty.to_token_stream(),
                        old_method.sig.ident
                    ),
                    suggested_fix: None,
                });
            }
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_detect_function_signature_change() {
        let old_code: File = parse_quote! {
            pub fn test_function(x: i32) -> String {
                "test".to_string()
            }
        };

        let new_code: File = parse_quote! {
            pub fn test_function(x: i32, y: i32) -> String {
                "test".to_string()
            }
        };

        let changes = compare_files(&old_code, &new_code);
        assert!(!changes.is_empty());
        assert!(matches!(
            changes[0].kind,
            ChangeKind::SignatureChanged { .. }
        ));
    }

    #[test]
    fn test_detect_removed_public_field() {
        let old_code: File = parse_quote! {
            pub struct Test {
                pub field: i32,
            }
        };

        let new_code: File = parse_quote! {
            pub struct Test {
                field: i32,
            }
        };

        let changes = compare_files(&old_code, &new_code);
        assert!(!changes.is_empty());
        assert!(matches!(changes[0].kind, ChangeKind::MethodRemoved { .. }));
    }

    #[test]
    fn test_detect_visibility_reduction() {
        let old_code: File = parse_quote! {
            pub fn test_function() {}
        };

        let new_code: File = parse_quote! {
            fn test_function() {}
        };

        let changes = compare_files(&old_code, &new_code);
        assert!(!changes.is_empty());
        assert!(matches!(
            changes[0].kind,
            ChangeKind::VisibilityReduced { .. }
        ));
    }
}
