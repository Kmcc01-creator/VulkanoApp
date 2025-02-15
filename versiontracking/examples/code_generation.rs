use std::path::PathBuf;
use versiontracking::code_generator::{
    CodeGenerator, CodeTransformation, OptimizationGoal, PatternType, TransformationType,
};

fn main() -> std::io::Result<()> {
    // Initialize code generator
    let mut generator = CodeGenerator::new();

    // Set up transformation configuration
    let mut config = CodeTransformation::new();

    // Include specific items to transform
    config.include_struct("AsyncResource");
    config.include_trait("ResourceManager");

    // Add transformations
    config.add_transformation(TransformationType::AsyncifyFunction {
        add_runtime: true,
        preserve_sync_version: true,
    });

    config.add_transformation(TransformationType::OptimizeErrorHandling {
        use_anyhow: true,
        generate_custom_errors: false,
    });

    config.add_optimization(OptimizationGoal::Performance);
    config.add_optimization(OptimizationGoal::Safety);

    // Example source code to transform
    let source = r#"
        pub struct AsyncResource {
            data: Vec<u8>,
        }

        impl AsyncResource {
            fn load_data() -> Result<Vec<u8>, std::io::Error> {
                std::fs::read("data.bin")
            }

            fn process(&self) -> Result<(), Box<dyn std::error::Error>> {
                // Some processing
                Ok(())
            }
        }

        pub trait ResourceManager {
            fn initialize() -> Result<(), std::io::Error>;
            fn cleanup() -> Result<(), std::io::Error>;
        }
    "#;

    // Parse and analyze the code
    let ast = syn::parse_str(source)?;
    generator.analyze_module(&ast);

    // Generate transformed code
    let output = generator.generate_new_module(&config);
    println!("Generated Code:\n{}", output);

    // Generate a complete new crate
    versiontracking::code_generator::generate_new_crate(
        "src/examples",
        "target/generated_crate",
        config,
    )?;

    println!("\nCode generation completed successfully!");
    println!("Generated crate is available in target/generated_crate/");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_async_transformation() -> std::io::Result<()> {
        let mut generator = CodeGenerator::new();
        let mut config = CodeTransformation::new();

        config.add_transformation(TransformationType::AsyncifyFunction {
            add_runtime: true,
            preserve_sync_version: true,
        });

        let source = r#"
            fn blocking_operation() -> Result<String, std::io::Error> {
                std::fs::read_to_string("test.txt")
            }
        "#;

        let ast = syn::parse_str(source).unwrap();
        generator.analyze_module(&ast);

        let output = generator.generate_new_module(&config);
        let generated = output.to_string();

        assert!(generated.contains("async fn"));
        assert!(generated.contains("_sync"));
        Ok(())
    }

    #[test]
    fn test_error_handling_transformation() -> std::io::Result<()> {
        let mut generator = CodeGenerator::new();
        let mut config = CodeTransformation::new();

        config.add_transformation(TransformationType::OptimizeErrorHandling {
            use_anyhow: true,
            generate_custom_errors: false,
        });

        let source = r#"
            fn process() -> Result<(), std::io::Error> {
                Ok(())
            }
        "#;

        let ast = syn::parse_str(source).unwrap();
        generator.analyze_module(&ast);

        let output = generator.generate_new_module(&config);
        let generated = output.to_string();

        assert!(generated.contains("anyhow::Result"));
        Ok(())
    }
}
