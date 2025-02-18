# Contributing Guide 🤝

## Getting Started

1. **Fork and Clone**

```bash
git clone https://github.com/yourusername/game-engine.git
cd game-engine
cargo build
```

2. **Set Up Development Environment**

- Install Rust (see [Getting Started](GettingStarted.md))
- Install Vulkan SDK
- Configure your IDE (we recommend VS Code with rust-analyzer)

## Development Workflow

### 1. Choose an Issue

- Check [open issues](https://github.com/username/game-engine/issues)
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

#### Documentation

- Update relevant documentation
- Add inline documentation for public APIs
- Include examples for new features

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

## Project Structure

See [Directory Structure](DirectoryStructure.md) for detailed layout.

```
src/
├── core/       # Core systems
├── graphics/   # Graphics pipeline
├── physics/    # Physics engine
└── scene/      # Scene management
```

## Release Process

1. **Version Bump**

```toml
# Cargo.toml
[package]
name = "game-engine"
version = "0.2.0"  # Update version
```

2. **Update Changelog**

```markdown
# CHANGELOG.md

## [0.2.0] - 2025-02-18

### Added

- PBR material support
- Improved physics collision

### Fixed

- Sphere collision detection
```

3. **Release Tags**

```bash
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0
```

## Communication

- **Issues**: Bug reports and feature requests
- **Discussions**: Design proposals and questions
- **Discord**: Real-time communication
- **Pull Requests**: Code review discussions

## Resources

- [Technical Reference](TechnicalReference.md)
- [Architecture Overview](../../docs/Architecture.md)
- [API Documentation](https://docs.example.com)

## Code of Conduct

We follow a standard code of conduct promoting:

- Inclusive environment
- Respectful communication
- Professional interaction
- Constructive feedback

## License

By contributing, you agree that your contributions will be licensed under the project's MIT License.

## Questions?

- Check our [Documentation](../../docs)
- Join our [Discord](https://discord.gg/example)
- Open a [Discussion](https://github.com/username/game-engine/discussions)

Thank you for contributing! 🎮
