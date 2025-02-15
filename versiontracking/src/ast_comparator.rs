use std::collections::HashMap;
use syn::{visit::Visit, File, ItemFn, ItemImpl, ItemStruct, ItemTrait, Signature};

#[derive(Debug)]
pub enum BreakingChange {
    FunctionSignatureChanged {
        name: String,
        old_sig: String,
        new_sig: String,
    },
    StructFieldRemoved {
        struct_name: String,
        field_name: String,
    },
    StructFieldTypeChanged {
        struct_name: String,
        field_name: String,
        old_type: String,
        new_type: String,
    },
    TraitImplementationRemoved {
        type_name: String,
        trait_name: String,
    },
    TraitMethodChanged {
        trait_name: String,
        method_name: String,
        details: String,
    },
}

#[derive(Default)]
struct AstData {
    functions: HashMap<String, Signature>,
    structs: HashMap<String, ItemStruct>,
    traits: HashMap<String, ItemTrait>,
    implementations: HashMap<String, Vec<ItemImpl>>,
}

#[derive(Default)]
pub struct ChangeTracker {
    pub changes: Vec<BreakingChange>,
}

impl ChangeTracker {
    fn add_change(&mut self, change: BreakingChange) {
        self.changes.push(change);
    }
}

struct DataCollector {
    data: AstData,
}

impl DataCollector {
    fn new() -> Self {
        Self {
            data: AstData::default(),
        }
    }
}

impl Visit<'_> for DataCollector {
    fn visit_item_fn(&mut self, func: &ItemFn) {
        let name = func.sig.ident.to_string();
        self.data.functions.insert(name, func.sig.clone());
        syn::visit::visit_item_fn(self, func);
    }

    fn visit_item_struct(&mut self, item_struct: &ItemStruct) {
        let name = item_struct.ident.to_string();
        self.data.structs.insert(name, item_struct.clone());
        syn::visit::visit_item_struct(self, item_struct);
    }

    fn visit_item_trait(&mut self, item_trait: &ItemTrait) {
        let name = item_trait.ident.to_string();
        self.data.traits.insert(name, item_trait.clone());
        syn::visit::visit_item_trait(self, item_trait);
    }

    fn visit_item_impl(&mut self, item_impl: &ItemImpl) {
        let type_name = format!("{:?}", item_impl.self_ty);
        self.data
            .implementations
            .entry(type_name)
            .or_default()
            .push(item_impl.clone());
        syn::visit::visit_item_impl(self, item_impl);
    }
}

fn compare_signatures(old_sig: &Signature, new_sig: &Signature) -> bool {
    if format!("{:?}", old_sig.output) != format!("{:?}", new_sig.output) {
        return false;
    }

    if old_sig.inputs.len() != new_sig.inputs.len() {
        return false;
    }

    old_sig
        .inputs
        .iter()
        .zip(new_sig.inputs.iter())
        .all(|(old, new)| format!("{:?}", old) == format!("{:?}", new))
}

fn find_breaking_changes(old_data: AstData, new_data: AstData) -> ChangeTracker {
    let mut tracker = ChangeTracker::default();

    // Compare functions
    for (name, old_sig) in old_data.functions {
        if let Some(new_sig) = new_data.functions.get(&name) {
            if !compare_signatures(&old_sig, new_sig) {
                tracker.add_change(BreakingChange::FunctionSignatureChanged {
                    name,
                    old_sig: format!("{:?}", old_sig),
                    new_sig: format!("{:?}", new_sig),
                });
            }
        }
    }

    // Compare structs
    for (name, old_struct) in old_data.structs {
        if let Some(new_struct) = new_data.structs.get(&name) {
            let old_fields: HashMap<_, _> = old_struct
                .fields
                .iter()
                .filter_map(|f| Some((f.ident.as_ref()?.to_string(), format!("{:?}", f.ty))))
                .collect();

            let new_fields: HashMap<_, _> = new_struct
                .fields
                .iter()
                .filter_map(|f| Some((f.ident.as_ref()?.to_string(), format!("{:?}", f.ty))))
                .collect();

            // Check for removed fields
            for field_name in old_fields.keys() {
                if !new_fields.contains_key(field_name) {
                    tracker.add_change(BreakingChange::StructFieldRemoved {
                        struct_name: name.clone(),
                        field_name: field_name.clone(),
                    });
                }
            }

            // Check for type changes
            for (field_name, old_type) in old_fields {
                if let Some(new_type) = new_fields.get(&field_name) {
                    if old_type != *new_type {
                        tracker.add_change(BreakingChange::StructFieldTypeChanged {
                            struct_name: name.clone(),
                            field_name,
                            old_type,
                            new_type: new_type.clone(),
                        });
                    }
                }
            }
        }
    }

    // Compare traits
    for (name, old_trait) in old_data.traits {
        if let Some(new_trait) = new_data.traits.get(&name) {
            let old_methods: HashMap<_, _> = old_trait
                .items
                .iter()
                .filter_map(|item| {
                    if let syn::TraitItem::Fn(m) = item {
                        Some((m.sig.ident.to_string(), format!("{:?}", m.sig)))
                    } else {
                        None
                    }
                })
                .collect();

            let new_methods: HashMap<_, _> = new_trait
                .items
                .iter()
                .filter_map(|item| {
                    if let syn::TraitItem::Fn(m) = item {
                        Some((m.sig.ident.to_string(), format!("{:?}", m.sig)))
                    } else {
                        None
                    }
                })
                .collect();

            for (method_name, old_sig) in old_methods {
                match new_methods.get(&method_name) {
                    Some(new_sig) => {
                        if old_sig != *new_sig {
                            tracker.add_change(BreakingChange::TraitMethodChanged {
                                trait_name: name.clone(),
                                method_name,
                                details: format!(
                                    "Signature changed from {} to {}",
                                    old_sig, new_sig
                                ),
                            });
                        }
                    }
                    None => {
                        tracker.add_change(BreakingChange::TraitMethodChanged {
                            trait_name: name.clone(),
                            method_name,
                            details: "Method removed".to_string(),
                        });
                    }
                }
            }
        }
    }

    // Compare implementations
    for (type_name, old_impls) in old_data.implementations {
        if let Some(new_impls) = new_data.implementations.get(&type_name) {
            for old_impl in &old_impls {
                if let Some(trait_path) = &old_impl.trait_ {
                    let trait_name = format!("{:?}", trait_path.1);

                    let found = new_impls.iter().any(|new_impl| {
                        new_impl
                            .trait_
                            .as_ref()
                            .map_or(false, |t| format!("{:?}", t.1) == trait_name)
                    });

                    if !found {
                        tracker.add_change(BreakingChange::TraitImplementationRemoved {
                            type_name: type_name.clone(),
                            trait_name,
                        });
                    }
                }
            }
        }
    }

    tracker
}

pub fn compare_asts(old_ast: &File, new_ast: &File) -> ChangeTracker {
    let mut old_collector = DataCollector::new();
    old_collector.visit_file(old_ast);

    let mut new_collector = DataCollector::new();
    new_collector.visit_file(new_ast);

    find_breaking_changes(old_collector.data, new_collector.data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_function_signature_changes() {
        let old_code = quote! {
            fn test_fn(x: i32) -> String {
                x.to_string()
            }
        };

        let new_code = quote! {
            fn test_fn(x: i64) -> String {
                x.to_string()
            }
        };

        let old_ast = parse_quote!(#old_code);
        let new_ast = parse_quote!(#new_code);

        let changes = compare_asts(&old_ast, &new_ast);
        assert!(!changes.changes.is_empty());
    }
}
