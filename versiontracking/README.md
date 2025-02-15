# Version Tracking and Code Generation Tools

A collection of tools for analyzing, transforming, and generating Rust code with a focus on API compatibility and code patterns.

## Project Structure

```
versiontracking/
├── crates/
│   ├── ast-analyzer/        # Core AST analysis functionality
│   │   ├── src/
│   │   │   ├── comparison.rs    # AST comparison for breaking changes
│   │   │   ├── extraction.rs    # Extract info from AST
│   │   │   ├── pattern.rs       # Pattern detection
│   │   │   ├── visitor.rs       # AST traversal
│   │   │   └── lib.rs          # Main analyzer interface
│   │   └── Cargo.toml
│   └── code-generator/      # Code generation functionality
│       ├── src/
│       │   ├── generator.rs     # Code generation orchestration
│       │   ├── transform.rs     # Code transformations
│       │   ├── template.rs      # Code templates
│       │   ├── patterns.rs      # Generation patterns
│       │   └── lib.rs          # Main generator interface
│       └── Cargo.toml
└── src/                     # Main library code
    ├── scripting/          # Scripting integration
    ├── version_management/ # Version tracking
    └── lib.rs             # Public API
```

## Features

- AST Analysis

  - Breaking change detection
  - Pattern recognition
  - Code metrics
  - Safety analysis

- Code Generation

  - Template-based generation
  - Pattern application
  - Code transformation
  - Documentation generation

- Version Management
  - API compatibility checking
  - Version bump analysis
  - Changelog generation

## Usage Examples

### Basic AST Analysis

```rust
use ast_analyzer::Analyzer;

let analyzer = Analyzer::new();
let analysis = analyzer.analyze(&source_file);

for pattern in analysis.patterns {
    println!("Found pattern: {:?}", pattern);
}
```

### Code Generation

```rust
use code_generator::{CodeGenerator, GenerationTarget, GeneratorConfig};

let mut generator = CodeGenerator::new(GeneratorConfig::default());

let target = GenerationTarget::Struct {
    name: "MyStruct".to_string(),
    fields: vec!["field1".to_string(), "field2".to_string()],
};

let code = generator.generate(target).await?;
println!("Generated code: {}", code);
```

### Apply Transformations

```rust
use code_generator::transform::{AsyncTransform, ErrorHandlingTransform};

generator.register_transform(Box::new(AsyncTransform::new()));
generator.register_transform(Box::new(ErrorHandlingTransform::new()));

let transformed = generator.generate_from_source(&source, target).await?;
```

### Use Generation Patterns

```rust
use code_generator::patterns::BuilderPattern;

let pattern = BuilderPattern::new(config);
let builder_code = pattern.generate(&context)?;
```

## Getting Started

1. Add dependencies to your `Cargo.toml`:

```toml
[dependencies]
ast-analyzer = { path = "crates/ast-analyzer" }
code-generator = { path = "crates/code-generator" }
```

2. Import and use the tools:

```rust
use ast_analyzer::Analyzer;
use code_generator::CodeGenerator;

// Set up analyzer
let analyzer = Analyzer::new();

// Set up generator
let generator = CodeGenerator::new(Default::default());

// Use them together
let analysis = analyzer.analyze(&source);
let code = generator.generate_from_analysis(analysis).await?;
```

## Configuration

The tools can be configured through their respective config structs:

```rust
let config = GeneratorConfig {
    transform_config: TransformConfig {
        async_conversion: true,
        error_handling: true,
        safety_checks: true,
        ..Default::default()
    },
    documentation: true,
    ..Default::default()
};
```

## Contributing

Contributions are welcome! Please see our contributing guidelines for more details.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
