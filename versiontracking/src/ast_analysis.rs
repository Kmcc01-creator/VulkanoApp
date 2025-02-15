use std::collections::{HashMap, HashSet};
use syn::{
    visit::Visit, Block, Expr, ExprCall, ExprMethodCall, File, FnArg, Generics, Ident, Item,
    ItemFn, ItemImpl, ItemStruct, ItemTrait, Path, Signature, Type, TypePath, WhereClause,
};

#[derive(Debug)]
pub enum SemanticDifference {
    TypeChange {
        old_type: String,
        new_type: String,
        is_breaking: bool,
    },
    GenericConstraintChange {
        param_name: String,
        old_constraints: String,
        new_constraints: String,
        is_breaking: bool,
    },
    LifetimeChange {
        old_lifetime: String,
        new_lifetime: String,
        is_breaking: bool,
    },
}

#[derive(Debug)]
pub struct ComparisonResult {
    pub has_changes: bool,
    pub is_breaking: bool,
    pub differences: Vec<SemanticDifference>,
}

#[derive(Debug)]
pub struct BlockingCall {
    pub span: proc_macro2::Span,
    pub function_name: String,
    pub is_io_blocking: bool,
    pub is_cpu_intensive: bool,
    pub estimated_duration: Option<std::time::Duration>,
}

pub struct SemanticAnalyzer {
    type_equivalence: HashMap<String, HashSet<String>>,
    known_blocking_functions: HashSet<String>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut type_equivalence = HashMap::new();
        let mut known_blocking_functions = HashSet::new();

        // Initialize type equivalences
        type_equivalence.insert(
            "Vec<T>".to_string(),
            vec!["&[T]", "Box<[T]>"]
                .into_iter()
                .map(String::from)
                .collect(),
        );

        // Initialize known blocking functions
        known_blocking_functions.extend(
            vec![
                "std::fs::read",
                "std::fs::read_to_string",
                "std::fs::write",
                "std::thread::sleep",
                "std::net::TcpStream::connect",
                "reqwest::blocking::get",
                "diesel::Connection::establish",
            ]
            .into_iter()
            .map(String::from),
        );

        Self {
            type_equivalence,
            known_blocking_functions,
        }
    }

    pub fn compare_signatures(&self, old_sig: &Signature, new_sig: &Signature) -> ComparisonResult {
        let mut differences = Vec::new();
        let mut is_breaking = false;

        // Compare return types
        if let Some(diff) = self.compare_types(
            old_sig.output.clone().into_token_stream().to_string(),
            new_sig.output.clone().into_token_stream().to_string(),
        ) {
            is_breaking |= diff.is_breaking;
            differences.push(diff);
        }

        // Compare parameters
        if old_sig.inputs.len() != new_sig.inputs.len() {
            is_breaking = true;
        } else {
            for (old_arg, new_arg) in old_sig.inputs.iter().zip(new_sig.inputs.iter()) {
                if let (FnArg::Typed(old_pat), FnArg::Typed(new_pat)) = (old_arg, new_arg) {
                    if let Some(diff) = self.compare_types(
                        old_pat.ty.clone().into_token_stream().to_string(),
                        new_pat.ty.clone().into_token_stream().to_string(),
                    ) {
                        is_breaking |= diff.is_breaking;
                        differences.push(diff);
                    }
                }
            }
        }

        // Compare generics
        if let Some(diff) = self.compare_generics(&old_sig.generics, &new_sig.generics) {
            is_breaking |= diff.is_breaking;
            differences.push(diff);
        }

        ComparisonResult {
            has_changes: !differences.is_empty(),
            is_breaking,
            differences,
        }
    }

    fn compare_types(&self, old_type: String, new_type: String) -> Option<SemanticDifference> {
        // If types are exactly the same, no difference
        if old_type == new_type {
            return None;
        }

        // Check type equivalence
        let is_breaking = !self.are_types_equivalent(&old_type, &new_type);

        Some(SemanticDifference::TypeChange {
            old_type,
            new_type,
            is_breaking,
        })
    }

    fn are_types_equivalent(&self, type1: &str, type2: &str) -> bool {
        if type1 == type2 {
            return true;
        }

        // Check if types are in equivalence set
        if let Some(equivalents) = self.type_equivalence.get(type1) {
            if equivalents.contains(type2) {
                return true;
            }
        }

        // Handle reference types
        if type1.starts_with('&') && type2.starts_with('&') {
            return self.are_types_equivalent(&type1[1..].trim_start(), &type2[1..].trim_start());
        }

        false
    }

    fn compare_generics(
        &self,
        old_generics: &Generics,
        new_generics: &Generics,
    ) -> Option<SemanticDifference> {
        let old_constraints = old_generics.clone().into_token_stream().to_string();
        let new_constraints = new_generics.clone().into_token_stream().to_string();

        if old_constraints == new_constraints {
            return None;
        }

        // Check if new constraints are more restrictive
        let is_breaking = self.are_constraints_more_restrictive(old_generics, new_generics);

        Some(SemanticDifference::GenericConstraintChange {
            param_name: "".to_string(), // TODO: Add parameter name
            old_constraints,
            new_constraints,
            is_breaking,
        })
    }

    fn are_constraints_more_restrictive(
        &self,
        old_generics: &Generics,
        new_generics: &Generics,
    ) -> bool {
        // TODO: Implement proper constraint analysis
        old_generics.params.len() < new_generics.params.len()
    }

    pub fn detect_blocking_calls(&self, block: &Block) -> Vec<BlockingCall> {
        let mut detector = BlockingCallDetector {
            analyzer: self,
            blocking_calls: Vec::new(),
        };
        detector.visit_block(block);
        detector.blocking_calls
    }
}

struct BlockingCallDetector<'a> {
    analyzer: &'a SemanticAnalyzer,
    blocking_calls: Vec<BlockingCall>,
}

impl<'a> Visit<'_> for BlockingCallDetector<'a> {
    fn visit_expr_call(&mut self, call: &ExprCall) {
        if let Expr::Path(path) = &*call.func {
            let func_name = path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if self.analyzer.known_blocking_functions.contains(&func_name) {
                self.blocking_calls.push(BlockingCall {
                    span: call.span(),
                    function_name: func_name,
                    is_io_blocking: true,
                    is_cpu_intensive: false,
                    estimated_duration: None,
                });
            }
        }
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &ExprMethodCall) {
        // Check if method is known to be blocking
        let method_name = call.method.to_string();
        if method_name.starts_with("blocking_")
            || method_name.contains("sync")
            || method_name.contains("wait")
        {
            self.blocking_calls.push(BlockingCall {
                span: call.span(),
                function_name: method_name,
                is_io_blocking: true,
                is_cpu_intensive: false,
                estimated_duration: None,
            });
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_type_comparison() {
        let analyzer = SemanticAnalyzer::new();

        // Test exact match
        assert!(analyzer.are_types_equivalent("Vec<T>", "Vec<T>"));

        // Test reference equivalence
        assert!(analyzer.are_types_equivalent("&str", "&str"));

        // Test known equivalences
        assert!(analyzer.are_types_equivalent("Vec<T>", "&[T]"));

        // Test non-equivalent types
        assert!(!analyzer.are_types_equivalent("String", "i32"));
    }

    #[test]
    fn test_blocking_call_detection() {
        let analyzer = SemanticAnalyzer::new();

        let block: Block = parse_quote!({
            std::fs::read_to_string("test.txt")?;
            std::thread::sleep(Duration::from_secs(1));
        });

        let blocking_calls = analyzer.detect_blocking_calls(&block);
        assert_eq!(blocking_calls.len(), 2);
    }
}
