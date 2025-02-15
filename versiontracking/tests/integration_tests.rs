use anyhow::Result;
use versiontracking::{
    scripting::{Interpreter, ScriptConfig},
    Analyzer, CodeGenerator, GenerationTarget, GeneratorConfig, Pattern, Transform,
    TransformConfig,
};

mod common;
use common::TestSetup;

#[tokio::test]
async fn test_full_workflow() -> Result<()> {
    let setup = TestSetup::new();

    // Test analysis
    let source = r#"
        pub fn blocking_function() {
            std::thread::sleep(std::time::Duration::from_secs(1));
            std::fs::read_to_string("test.txt").unwrap();
        }
    "#;

    let analyzer = Analyzer::new();
    let analysis = analyzer.analyze(&syn::parse_str(source)?);

    assert!(analysis.patterns.iter().any(|p| p.pattern == "blocking_io"));
    assert!(analysis
        .patterns
        .iter()
        .any(|p| p.pattern == "unsafe_unwrap"));

    // Test code generation
    let target = GenerationTarget::Function {
        name: "async_function".to_string(),
        is_async: true,
        is_unsafe: false,
    };

    let generator = CodeGenerator::new(GeneratorConfig::default());
    let code = generator.generate(target).await?;

    assert!(code.to_string().contains("async fn async_function"));

    // Test scripting
    let script = r#"
        analyze src/lib.rs {
            detect blocking_io
            detect unsafe_blocks
        }

        generate AsyncWrapper struct {
            field inner: String
        }

        apply_pattern async_wrapper AsyncWrapper {
            error_handling = "true"
        }
    "#;

    let mut interpreter = Interpreter::new();
    interpreter.execute_script(script).await?;

    Ok(())
}

#[tokio::test]
async fn test_pattern_combinations() -> Result<()> {
    let setup = TestSetup::new();

    let source = r#"
        pub struct Resource {
            handle: std::fs::File,
        }
    "#;

    // Apply multiple patterns
    let script = r#"
        apply_pattern resource_guard Resource {
            cleanup = "true"
        }

        apply_pattern builder Resource {
            validation = "true"
        }

        apply_pattern async_wrapper Resource {
            error_handling = "true"
        }
    "#;

    let mut interpreter = Interpreter::new();
    interpreter.execute_script(script).await?;

    // Verify results
    let analyzer = Analyzer::new();
    let analysis = analyzer.analyze(&syn::parse_str(source)?);

    assert!(analysis
        .patterns
        .iter()
        .any(|p| p.pattern == "resource_management"));

    Ok(())
}

#[tokio::test]
async fn test_breaking_changes() -> Result<()> {
    let old_code = r#"
        pub struct Config {
            pub field: String,
        }

        impl Config {
            pub fn new(field: String) -> Self {
                Self { field }
            }
        }
    "#;

    let new_code = r#"
        pub struct Config {
            field: String, // Changed visibility
        }

        impl Config {
            fn new(field: String) -> Self { // Changed visibility
                Self { field }
            }
        }
    "#;

    let analyzer = Analyzer::new();
    let changes = analyzer.compare_versions(&syn::parse_str(old_code)?, &syn::parse_str(new_code)?);

    assert!(changes.iter().any(|c| c.kind == "visibility_reduced"));

    Ok(())
}

#[tokio::test]
async fn test_transform_pipeline() -> Result<()> {
    let config = TransformConfig {
        async_conversion: true,
        error_handling: true,
        safety_checks: true,
        ..Default::default()
    };

    let mut generator = CodeGenerator::new(GeneratorConfig {
        transform_config: config.clone(),
        ..Default::default()
    });

    let source = r#"
        fn process() {
            let data = std::fs::read_to_string("test.txt").unwrap();
            println!("{}", data);
        }
    "#;

    // Generate with transforms
    let target = GenerationTarget::Function {
        name: "process".to_string(),
        is_async: true,
        is_unsafe: false,
    };

    let code = generator
        .generate_from_source(&syn::parse_str(source)?, target)
        .await?;
    let code_str = code.to_string();

    assert!(code_str.contains("async"));
    assert!(code_str.contains("Result"));

    Ok(())
}

#[tokio::test]
async fn test_config_handling() -> Result<()> {
    let config = ScriptConfig::default_config();
    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().join(".version-tracking.toml");

    // Save and load config
    config.save_to_file(&config_path)?;
    let loaded = ScriptConfig::from_file(&config_path)?;

    assert_eq!(
        loaded.generation.documentation,
        config.generation.documentation
    );

    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let result = Interpreter::new().execute_script("invalid { syntax").await;

    assert!(result.is_err());

    let result = CodeGenerator::new(Default::default())
        .generate(GenerationTarget::Function {
            name: "".to_string(), // Invalid name
            is_async: false,
            is_unsafe: false,
        })
        .await;

    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    use futures::future::join_all;

    let generator = CodeGenerator::new(Default::default());
    let targets = (0..10).map(|i| GenerationTarget::Function {
        name: format!("function_{}", i),
        is_async: true,
        is_unsafe: false,
    });

    let results = join_all(targets.map(|t| generator.generate(t))).await;

    assert!(results.into_iter().all(|r| r.is_ok()));

    Ok(())
}
