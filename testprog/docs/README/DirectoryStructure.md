# Project Directory Structure 📁

## Overview

```
testprog/
├── src/               # Source code
├── examples/          # Example applications
├── docs/             # Documentation
├── tests/            # Test suites
└── assets/           # Engine assets
```

## Source Code Structure 🔧

```
src/
├── core/             # Core engine systems
│   ├── mod.rs           # Module definition
│   ├── engine.rs        # Main engine implementation
│   ├── error.rs         # Error handling
│   ├── input.rs         # Input system
│   └── window.rs        # Window management
│
├── graphics/         # Graphics system
│   ├── mod.rs           # Module definition
│   ├── device.rs        # Device management
│   ├── pipeline.rs      # Graphics pipeline
│   ├── renderer.rs      # Main renderer
│   ├── shader.rs        # Shader management
│   ├── swapchain.rs     # Swapchain handling
│   └── vertex.rs        # Vertex definitions
│
├── physics/          # Physics engine
│   ├── mod.rs           # Module definition
│   ├── collider.rs      # Collision shapes
│   ├── physics_world.rs # Physics simulation
│   └── rigidbody.rs     # Rigid body dynamics
│
├── scene/           # Scene management
│   ├── mod.rs           # Module definition
│   ├── component.rs     # Component system
│   ├── entity.rs        # Entity management
│   ├── scene_graph.rs   # Scene hierarchy
│   ├── system.rs        # System implementation
│   └── world.rs         # World management
│
├── resource/        # Resource management
│   ├── mod.rs           # Module definition
│   ├── asset.rs         # Asset handling
│   ├── cache.rs         # Resource caching
│   └── loader.rs        # Resource loading
│
└── ui/             # User interface system
    ├── mod.rs           # Module definition
    ├── widget.rs        # UI widgets
    └── layout.rs        # Layout system
```

## Examples Directory 📚

```
examples/
├── basic_window.rs      # Basic window creation
├── event_loop.rs        # Event system demo
├── input_handling.rs    # Input system usage
├── physics_demo.rs      # Physics simulation
├── rendering/           # Graphics examples
│   ├── basic_triangle.rs
│   └── textured_cube.rs
├── scene_demo/         # Scene system examples
│   ├── components.rs
│   └── hierarchy.rs
└── ui_demo.rs          # UI system demo
```

## Documentation Structure 📖

```
docs/
├── README/            # Main documentation
│   ├── README.md         # Project overview
│   ├── GettingStarted.md # Setup guide
│   └── TechnicalReference.md # API reference
│
├── guides/           # Detailed guides
│   ├── architecture.md
│   ├── graphics.md
│   ├── physics.md
│   └── scripting.md
│
├── api/             # API documentation
│   ├── core.md
│   ├── graphics.md
│   ├── physics.md
│   └── scene.md
│
└── historic/        # Archived documentation
    └── old_versions/
```

## Tests Organization 🧪

```
tests/
├── integration/     # Integration tests
│   ├── graphics_tests.rs
│   ├── physics_tests.rs
│   └── scene_tests.rs
│
├── unit/           # Unit tests
│   ├── core_tests.rs
│   ├── math_tests.rs
│   └── utils_tests.rs
│
└── common/         # Shared test utilities
    ├── helpers.rs
    └── mock.rs
```

## Asset Organization 🎨

```
assets/
├── shaders/         # Shader files
│   ├── compiled/       # Compiled SPIR-V
│   └── source/        # Shader source
│
├── models/          # 3D models
│   ├── primitives/    # Basic shapes
│   └── characters/    # Character models
│
├── textures/       # Texture files
│   ├── materials/     # Material textures
│   └── ui/           # UI textures
│
└── config/         # Configuration files
    ├── engine.toml    # Engine config
    └── graphics.toml  # Graphics settings
```

## Key Files 📄

### Root Level

- `Cargo.toml` - Project configuration and dependencies
- `README.md` - Project overview and documentation
- `LICENSE` - Project license
- `CONTRIBUTING.md` - Contribution guidelines

### Source Code

- `src/lib.rs` - Library entry point and exports
- `src/main.rs` - Binary entry point (if applicable)
- `src/core/mod.rs` - Core module organization
- `src/core/engine.rs` - Main engine implementation

### Configuration

- `assets/config/engine.toml` - Engine configuration
- `assets/config/graphics.toml` - Graphics settings
- `.cargo/config.toml` - Cargo configuration

## Note on Directory Organization

- Each major system has its own directory under `src/`
- Implementation files are kept separate from interface files
- Tests are co-located with source files where appropriate
- Documentation is organized by topic and type
- Examples demonstrate one concept per file

This structure promotes:

- Clear separation of concerns
- Easy navigation
- Logical grouping of related files
- Scalability for future additions
