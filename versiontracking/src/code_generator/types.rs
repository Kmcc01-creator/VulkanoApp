use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct ModuleStructure {
    pub functions: Vec<FunctionInfo>,
    pub structs: Vec<StructInfo>,
    pub traits: Vec<TraitInfo>,
    pub implementations: Vec<ImplInfo>,
    pub type_dependencies: HashMap<String, Vec<String>>,
}

#[derive(Debug)]
pub struct FunctionInfo {
    pub name: String,
    pub signature: String,
    pub dependencies: Vec<String>,
    pub is_public: bool,
    pub documentation: Option<String>,
}

#[derive(Debug)]
pub struct StructInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
    pub derives: Vec<String>,
    pub is_public: bool,
    pub documentation: Option<String>,
}

#[derive(Debug)]
pub struct FieldInfo {
    pub name: String,
    pub type_name: String,
    pub is_public: bool,
    pub documentation: Option<String>,
}

#[derive(Debug)]
pub struct TraitInfo {
    pub name: String,
    pub methods: Vec<TraitMethodInfo>,
    pub supertraits: Vec<String>,
    pub is_public: bool,
    pub documentation: Option<String>,
}

#[derive(Debug)]
pub struct TraitMethodInfo {
    pub name: String,
    pub signature: String,
    pub has_default_impl: bool,
    pub documentation: Option<String>,
}

#[derive(Debug)]
pub struct ImplInfo {
    pub target_type: String,
    pub trait_name: Option<String>,
    pub methods: Vec<FunctionInfo>,
}

#[derive(Debug)]
pub struct PatternMatch {
    pub pattern_type: PatternType,
    pub location: CodeLocation,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone)]
pub enum PatternType {
    AsyncAntiPattern,
    SuboptimalErrorHandling,
    ComplexLifetimes,
    UnsafeBlock,
    RedundantClone,
    LockContention,
    BoxedClosure,
    VecWithCapacity,
    StringWithCapacity,
}

#[derive(Debug, Clone)]
pub struct CodeLocation {
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub title: String,
    pub description: String,
    pub code: String,
    pub safety_impact: Option<String>,
    pub performance_impact: Option<String>,
}
