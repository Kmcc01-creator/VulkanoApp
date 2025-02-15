# Code Generation Patterns

This document describes the common patterns and best practices for using the code generation and AST analysis tools.

## AST Analysis Patterns

### Breaking Change Detection

The analyzer looks for several types of breaking changes:

```rust
// Before
pub fn process(data: &str) -> Result<String, Error> {
    // ...
}

// After - Breaking Changes:
fn process(data: &str) -> String {  // Visibility reduced, error handling removed
    // ...
}
```

### Pattern Recognition

Common patterns that are detected:

1. **Blocking Operations**

```rust
// Pattern: Blocking I/O
fn read_file() {
    std::fs::read_to_string("file.txt").unwrap();
}
```

2. **Resource Management**

```rust
// Pattern: Resource Leak Potential
struct Connection {
    socket: TcpStream,
}
// Missing Drop implementation
```

3. **Safety Issues**

```rust
// Pattern: Unsafe Usage
unsafe {
    let ptr = std::ptr::null();
    *ptr;  // Dangerous!
}
```

## Code Generation Patterns

### Builder Pattern

```rust
// Input:
struct Config {
    host: String,
    port: u16,
}

// Generated:
impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

pub struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
}

impl ConfigBuilder {
    pub fn host(mut self, value: impl Into<String>) -> Self {
        self.host = Some(value.into());
        self
    }

    pub fn port(mut self, value: u16) -> Self {
        self.port = Some(value);
        self
    }

    pub fn build(self) -> Result<Config, &'static str> {
        Ok(Config {
            host: self.host.ok_or("host is required")?,
            port: self.port.ok_or("port is required")?,
        })
    }
}
```

### Async Wrapper Pattern

```rust
// Input:
fn process_data(data: &[u8]) -> Result<Vec<u8>, Error> {
    // ...
}

// Generated:
async fn process_data_async(data: &[u8]) -> Result<Vec<u8>, Error> {
    tokio::task::spawn_blocking(move || {
        process_data(data)
    }).await?
}
```

### Error Handling Pattern

```rust
// Input:
fn fallible_operation() -> String {
    // ...
}

// Generated:
fn fallible_operation() -> Result<String, Error> {
    let result = std::fs::read_to_string("file.txt")
        .context("failed to read file")?;
    Ok(result)
}
```

## Template Usage

### Basic Template

```rust
let template = Template::new("struct")
    .with_generator(|ctx| {
        let name = ctx.get_string("name")?;
        let fields = ctx.get_list("fields")?;

        quote! {
            pub struct #name {
                #(pub #fields: String,)*
            }
        }
    });
```

### Customizing Templates

```rust
let mut context = TemplateContext::new();
context.set("struct_name", "MyConfig");
context.set("fields", vec!["host", "port"]);
context.set("visibility", "pub(crate)");

let code = template.render(&context)?;
```

## Transformation Pipeline

### Setting Up Transforms

```rust
let mut generator = CodeGenerator::new(config);

// Add transforms in order
generator.register_transform(Box::new(AsyncTransform::new()));
generator.register_transform(Box::new(ErrorHandlingTransform::new()));
generator.register_transform(Box::new(SafetyTransform::new()));
```

### Custom Transforms

```rust
#[async_trait]
impl Transform for MyTransform {
    fn name(&self) -> &str {
        "my_transform"
    }

    async fn apply(&self, code: TokenStream, context: &GenerationContext) -> Result<TokenStream> {
        // Transform the code
        Ok(transformed_code)
    }
}
```

## Best Practices

1. **Analysis First**: Always analyze source code before generating new code to understand the context.

   ```rust
   let analysis = analyzer.analyze(&source);
   let code = generator.generate_from_analysis(analysis)?;
   ```

2. **Pattern Composition**: Combine multiple patterns for complex generations.

   ```rust
   generator.register_template("composite", Template::new()
       .with_pattern(BuilderPattern::new())
       .with_pattern(AsyncWrapperPattern::new()));
   ```

3. **Progressive Enhancement**: Apply transforms in order of importance.

   ```rust
   // Order matters:
   generator.register_transform(Box::new(SafetyTransform::new()));  // First
   generator.register_transform(Box::new(AsyncTransform::new()));   // Then async
   generator.register_transform(Box::new(ErrorHandlingTransform::new())); // Finally
   ```

4. **Context Awareness**: Use the analysis results to inform generation.
   ```rust
   let context = GenerationContext {
       target,
       patterns: analysis.patterns,
       ..Default::default()
   };
   ```

## Future Patterns

We're working on additional patterns:

1. **State Machine Generation**
2. **Event Handler Generation**
3. **API Client Generation**
4. **Database Schema Generation**

Stay tuned for updates!
