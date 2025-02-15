use std::path::PathBuf;

pub mod error_handling;
pub mod patterns;
pub mod transforms;
pub mod types;

pub use error_handling::{ErrorHandlingConfig, ErrorTransformer};
pub use transforms::{CodeTransformer, TransformationType};

#[derive(Debug)]
pub struct GeneratedCode {
    pub source: String,
    pub path: PathBuf,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub enum GenerationError {
    InvalidInput(String),
    TransformationFailed(String),
    ValidationFailed(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for GenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Self::TransformationFailed(msg) => write!(f, "Transformation failed: {}", msg),
            Self::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            Self::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for GenerationError {}

impl From<std::io::Error> for GenerationError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err)
    }
}

pub struct CodeGenerator {
    transformer: CodeTransformer,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            transformer: CodeTransformer::new(),
        }
    }

    pub fn add_transformation(&mut self, transformation: TransformationType) {
        self.transformer.add_transformation(transformation);
    }

    pub fn generate(&self, input: &str) -> Result<GeneratedCode, GenerationError> {
        let ast = syn::parse_str(input).map_err(|e| {
            GenerationError::InvalidInput(format!("Failed to parse input code: {}", e))
        })?;

        let transformed = self
            .transformer
            .transform_ast(&mut ast)
            .map_err(|e| GenerationError::TransformationFailed(e.to_string()))?;

        Ok(GeneratedCode {
            source: transformed.to_string(),
            path: PathBuf::new(),
            warnings: Vec::new(),
        })
    }

    pub fn set_output_path(&mut self, path: PathBuf) {
        // TODO: Implement path handling
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_generation() {
        let input = r#"
            fn blocking_function() -> Result<String, std::io::Error> {
                std::fs::read_to_string("test.txt")
            }
        "#;

        let mut generator = CodeGenerator::new();
        generator.add_transformation(TransformationType::AsyncifyFunction {
            add_runtime: true,
            preserve_sync_version: false,
        });

        let result = generator.generate(input).unwrap();
        assert!(result.source.contains("async"));
    }
}
