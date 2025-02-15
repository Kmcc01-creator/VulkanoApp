# Scripting System Reference

This document provides a comprehensive reference for the version tracking tools scripting system.

## Script Syntax

### Basic Structure

Scripts are composed of commands, each with their own block of options:

```rust
command_name target_name {
    option1 = "value1"
    option2 = "value2"
}
```

### Comments

```rust
// Single line comment
/* Multi-line
   comment */
```

## Commands

### 1. Generate Command

Generates new code structures.

```rust
generate MyStruct struct {
    field name: String
    field age: u32
    visibility = "pub"
    derive = "Debug, Clone"
}
```

Options:

- `visibility`: Visibility level (`pub`, `pub(crate)`, etc.)
- `derive`: Comma-separated list of traits to derive
- `async`: Whether to generate async code
- `doc`: Whether to generate documentation

### 2. Apply Pattern Command

Applies a code generation pattern.

```rust
apply_pattern builder MyStruct {
    validation = "true"
    async = "true"
    visibility = "pub"
}
```

Available Patterns:

- `builder`: Builder pattern
- `async_wrapper`: Async conversion
- `resource_guard`: Resource management
- `state_machine`: State machine
- `rest_client`: API client
- `metrics`: Metrics collection
- `thread_safe`: Thread safety wrapper

### 3. Analyze Command

Analyzes code for patterns and issues.

```rust
analyze src/lib.rs {
    detect unsafe_blocks
    detect blocking_io
    check_safety = true
    strict = true
}
```

Detection Options:

- `unsafe_blocks`: Find unsafe code
- `blocking_io`: Find blocking operations
- `thread_safety`: Check thread safety
- `resource_leaks`: Find potential resource leaks
- `performance_issues`: Identify performance problems

### 4. Configure Command

Configures tool behavior.

```rust
configure analysis {
    check_safety = "true"
    detect_patterns = "true"
}

configure generation {
    documentation = "true"
    optimization = "size"
}
```

Configuration Sections:

1. Analysis

   - `check_safety`: Enable safety checks
   - `detect_patterns`: Enable pattern detection
   - `async_aware`: Enable async code analysis

2. Generation
   - `documentation`: Generate documentation
   - `optimization`: Optimization level
   - `safety_checks`: Add runtime checks

## Pattern Reference

### Builder Pattern

```rust
apply_pattern builder MyStruct {
    validation = "true"  // Add validation
    doc = "true"        // Add documentation
    async = "false"     // Synchronous builder
}
```

### Resource Guard

```rust
apply_pattern resource_guard Resource {
    cleanup = "true"     // Add cleanup code
    error_handling = "true"  // Add error handling
}
```

### Async Wrapper

```rust
apply_pattern async_wrapper {
    target = "blocking_functions"
    error_handling = "true"
}
```

### State Machine

```rust
apply_pattern state_machine State {
    async = "true"
    transitions = "true"
    visualization = "true"
}
```

## Example Workflows

### 1. API Client Generation

```rust
// Generate basic structure
generate ApiClient struct {
    field base_url: String
    field timeout: Duration
}

// Add builder pattern
apply_pattern builder ApiClient {
    validation = "true"
}

// Make it async
apply_pattern async_wrapper ApiClient {
    error_handling = "true"
}

// Add metrics
apply_pattern metrics ApiClient {
    prometheus = "true"
}
```

### 2. Resource Management

```rust
// Generate resource type
generate Resource struct {
    field handle: RawHandle
}

// Add safety wrapper
apply_pattern resource_guard Resource {
    cleanup = "true"
}

// Analyze implementation
analyze src/resource.rs {
    detect resource_leaks
    detect thread_safety
}
```

## Best Practices

1. **Order of Operations**

   - Generate base structures first
   - Apply patterns
   - Run analysis
   - Configure as needed

2. **Pattern Combinations**

   ```rust
   // Good: Logical order
   apply_pattern builder MyStruct { ... }
   apply_pattern async_wrapper MyStruct { ... }
   apply_pattern metrics MyStruct { ... }
   ```

3. **Safety Checks**

   ```rust
   // Always run safety analysis after changes
   analyze src/ {
       detect all
       strict = "true"
   }
   ```

4. **Documentation**
   ```rust
   configure documentation {
       style = "full"
       examples = "true"
   }
   ```

## Error Handling

Common error messages and solutions:

1. **Pattern Not Found**

   ```
   Error: Pattern 'unknown_pattern' not found
   ```

   Solution: Check available patterns in documentation

2. **Invalid Syntax**

   ```
   Error: Expected opening brace
   ```

   Solution: Check command block syntax

3. **Missing Required Options**
   ```
   Error: Required option 'target' missing
   ```
   Solution: Add all required options

## Performance Tips

1. Use targeted analysis:

   ```rust
   analyze src/core/ {
       detect performance_issues
   }
   ```

2. Configure optimization:

   ```rust
   configure transforms {
       optimization = "speed"
   }
   ```

3. Batch related commands:
   ```rust
   // Good: Related operations together
   generate MyStruct struct { ... }
   apply_pattern builder MyStruct { ... }
   analyze generated/my_struct.rs { ... }
   ```

## Debugging

Add debug output:

```rust
configure debug {
    level = "verbose"
    output = "debug.log"
}
```

## Script Examples

See [examples/scripts/examples.script](../examples/scripts/examples.script) for more complete examples.
