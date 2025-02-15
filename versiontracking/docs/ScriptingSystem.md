# Scripting System Integration

This document explains how the scripting system integrates with the code generation and analysis tools to enable dynamic code manipulation.

## Overview

The scripting system provides a high-level interface for:

- Defining code transformations
- Creating custom generation patterns
- Automating analysis tasks
- Configuring code generation pipelines

## Scripting Language

The scripting system uses a domain-specific language (DSL) designed for code manipulation:

```rust
// Example script
transform MyStruct {
    // Add builder pattern
    apply_pattern builder;

    // Add async capabilities
    apply_pattern async_wrapper;

    // Add custom fields
    add_field timestamp: DateTime<Utc>;
    add_field metadata: HashMap<String, String>;
}

// Configure analysis
analyze {
    check_breaking_changes = true;
    detect_patterns = true;
    safety_checks = true;
}
```

## Integration Points

### 1. AST Analysis Integration

```rust
// Script
analyze MyStruct {
    detect_pattern resource_management;
    detect_pattern async_blocking;
}

// Generated Analysis Code
let analyzer = Analyzer::new()
    .with_pattern(ResourcePattern::new())
    .with_pattern(AsyncBlockingPattern::new());

let analysis = analyzer.analyze(&source);
```

### 2. Code Generation Integration

```rust
// Script
generate Api {
    template rest_client;
    options {
        async = true;
        error_handling = true;
    }
}

// Generated Generator Code
let generator = CodeGenerator::new(GeneratorConfig {
    transform_config: TransformConfig {
        async_conversion: true,
        error_handling: true,
        ..Default::default()
    },
    ..Default::default()
});
```

### 3. Transform Pipeline Integration

```rust
// Script
pipeline ApiTransform {
    stage 1 {
        apply_pattern builder;
        apply_pattern async;
    }
    stage 2 {
        apply_safety_checks;
        add_documentation;
    }
}

// Generated Pipeline Code
let mut pipeline = TransformPipeline::new();
pipeline.add_stage(vec![
    Box::new(BuilderPattern::new()),
    Box::new(AsyncPattern::new()),
]);
pipeline.add_stage(vec![
    Box::new(SafetyTransform::new()),
    Box::new(DocTransform::new()),
]);
```

## Custom Pattern Definition

```rust
// Script
define_pattern ResourceGuard {
    match {
        struct $name {
            $fields...
        }
    }

    generate {
        impl Drop for $name {
            fn drop(&mut self) {
                // Generated cleanup code
            }
        }
    }
}

// Usage
apply_pattern ResourceGuard to MyStruct;
```

## Script Configuration

### Project-Level Configuration

```toml
# .version-tracking.toml
[scripting]
patterns_dir = "scripts/patterns"
templates_dir = "scripts/templates"
auto_apply = true

[analysis]
check_breaking_changes = true
detect_patterns = true
safety_checks = true

[generation]
documentation = true
error_handling = true
async_support = true
```

### Pattern-Specific Configuration

```rust
// Script
configure BuilderPattern {
    visibility = "pub(crate)";
    validation = true;
    doc_comments = true;
}
```

## Common Use Cases

### 1. API Version Management

```rust
// Script
version_bump Api {
    if has_breaking_changes {
        bump_major;
    } else if has_new_features {
        bump_minor;
    } else {
        bump_patch;
    }

    generate_changelog;
}
```

### 2. Code Modernization

```rust
// Script
modernize Code {
    replace_pattern old_error_handling with anyhow;
    replace_pattern blocking_io with tokio;
    apply_pattern async_trait;
}
```

### 3. Safety Enforcement

```rust
// Script
enforce_safety {
    check unsafe_blocks;
    check resource_leaks;
    check threading_issues;

    on_violation {
        generate_fix;
        add_warning;
    }
}
```

## Debugging and Testing

### Script Testing

```rust
// test.script
#[test]
pattern BuilderPattern {
    input = {
        struct Test {
            field: String
        }
    }

    expect = {
        // Expected generated code
        struct TestBuilder {
            field: Option<String>
        }
    }
}
```

### Debug Output

```rust
// Script
debug_mode {
    print_ast;
    print_transformations;
    print_generated_code;
}
```

## Best Practices

1. **Modular Scripts**: Break down complex transformations into reusable patterns

2. **Version Control**: Keep scripts in version control alongside code

3. **Testing**: Write tests for custom patterns and transformations

4. **Documentation**: Document pattern purposes and side effects

5. **Progressive Enhancement**: Apply transformations in logical order

## Limitations

1. Complex type inference may require manual hints
2. Some patterns may need manual adjustment
3. Generated code may need manual optimization
4. Not all patterns can be automatically applied

## Future Development

1. Enhanced pattern matching capabilities
2. More built-in patterns and transformations
3. Better IDE integration
4. Performance optimizations
5. Extended scripting DSL features
