# Contributing Guide 🤝

## Quick Navigation

- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Code Guidelines](#code-guidelines)
- [Documentation](#documentation)
- Back to [Main Documentation](README.md)

## Getting Started

1. **Fork and Clone**

```bash
git clone https://github.com/yourusername/game-engine.git
cd game-engine
cargo build
```

2. **Set Up Development Environment**

- Install Rust (see [Getting Started Guide](GettingStarted.md))
- Install Vulkan SDK
- Configure your IDE (we recommend VS Code with rust-analyzer)

## Development Workflow

### 1. Choose an Issue

- Check open issues on GitHub
- Look for `good first issue` labels for beginners
- Comment on the issue you'd like to work on

### 2. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-fix-name
```

### 3. Development Guidelines

#### Code Style

```rust
// Use clear, descriptive names
pub struct RenderPipeline {
    device: Arc<Device>,
    pipeline: Arc<GraphicsPipeline>,
}

// Document public APIs
/// Creates a new render pipeline
///
/// # Arguments
/// * `device` - The Vulkan device
/// * `config` - Pipeline configuration
///
/// # Returns
/// A new render pipeline instance
pub fn new(device: Arc<Device>, config: PipelineConfig) -> Result<Self, Error> {
    // Implementation
}
```

#### Testing

- Write unit tests for new functionality
- Update existing tests as needed
- Add integration tests for system interactions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        // Test implementation
    }
}
```

### 4. Commit Guidelines

```bash
# Good commit messages
git commit -m "feat(graphics): add support for PBR materials"
git commit -m "fix(physics): correct collision detection for spheres"
git commit -m "docs: update API reference for scene system"
```

Use conventional commit types:

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Formatting changes
- `refactor`: Code restructuring
- `test`: Adding/updating tests
- `chore`: Maintenance tasks

### 5. Pull Request Process

1. **Update Your Branch**

```bash
git fetch origin
git rebase origin/main
```

2. **Run Checks**

```bash
# Run tests
cargo test

# Run clippy
cargo clippy -- -D warnings

# Format code
cargo fmt
```

3. **Create Pull Request**

- Use the PR template
- Link related issues
- Provide clear description
- Add screenshots/videos if relevant

## Documentation

### Required Documentation

1. **Code Documentation**

   - Document all public APIs
   - Include usage examples
   - Explain complex algorithms
   - Add performance notes

2. **Testing Documentation**

   - Document test cases
   - Explain test setup
   - Include test data sources

3. **Update Guides**
   - Update [Technical Reference](TechnicalReference.md) if needed
   - Add examples to [Getting Started](GettingStarted.md) if relevant
   - Update [README](README.md) for major changes

## Project Structure

See [Directory Structure](DirectoryStructure.md) for detailed project layout.

## Review Process

### What We Look For

- Code quality and style
- Test coverage
- Documentation
- Performance implications
- Breaking changes

### After Review

1. Address review comments
2. Update your branch
3. Request re-review if needed

## Release Process

1. **Version Bump**

```toml
# Cargo.toml
[package]
name = "game-engine"
version = "0.2.0"  # Update version
```

2. **Update CHANGELOG**

```markdown
## [0.2.0] - 2025-02-18

### Added

- PBR material support
- Improved physics collision

### Fixed

- Sphere collision detection
```

## Questions?

- Check our [Documentation Index](index.md)
- Review [Technical Reference](TechnicalReference.md)
- See [Getting Started Guide](GettingStarted.md)

---

<div align="center">

[Back to README](README.md) | [Getting Started](GettingStarted.md) | [Technical Reference](TechnicalReference.md)

</div>
