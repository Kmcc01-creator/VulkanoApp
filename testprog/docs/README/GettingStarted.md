# Getting Started with the Engine 🚀

## Prerequisites

- Rust 2021 Edition or later
- Vulkan SDK 1.3+
- CMake 3.15+
- Git

## Quick Navigation

- [Basic Setup](#installation)
- [First Steps](#first-steps)
- [Example Code](#creating-your-first-application)
- [Core Concepts](#core-concepts)
- Back to [Main Documentation](README.md)

## Installation

1. **Install Rust**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **Install Vulkan SDK**

```bash
# Linux
wget -qO- https://packages.lunarg.com/lunarg-signing-key-pub.asc | sudo apt-key add -
sudo wget -qO /etc/apt/sources.list.d/lunarg-vulkan.list \
    https://packages.lunarg.com/vulkan/lunarg-vulkan.list
sudo apt update
sudo apt install vulkan-sdk

# Windows
# Download and install from https://vulkan.lunarg.com/
```

3. **Clone the Repository**

```bash
git clone https://github.com/username/game-engine.git
cd game-engine
```

4. **Build the Project**

```bash
cargo build
```

## First Steps

### 1. Run the Examples

```bash
# Basic window creation
cargo run --example basic_window

# Physics demo
cargo run --example physics_demo

# UI system demo
cargo run --example ui_demo
```

### 2. Create Your First Application

```rust
use engine::{
    Window,
    Scene,
    Entity,
    components::{Transform, Mesh, Material},
};

fn main() {
    // Create window and scene
    let mut window = Window::new();
    let mut scene = Scene::new();

    // Create a basic cube
    let cube = Entity::builder()
        .with_component(Transform::default())
        .with_component(Mesh::cube())
        .with_component(Material::default())
        .build();

    // Add to scene
    scene.add_entity(cube);

    // Run the application
    window.run(move |frame| {
        scene.update();
        scene.render(frame);
    });
}
```

## Core Concepts

### 1. Scene Management

```rust
// Creating a scene
let mut scene = Scene::new();

// Adding entities
let entity = Entity::new();
scene.add_entity(entity);

// Querying entities
let entities = scene.query::<(&Transform, &Mesh)>();
```

### 2. Graphics Pipeline

```rust
// Creating a material
let material = Material::builder()
    .with_shader("shaders/pbr.vert.spv")
    .with_shader("shaders/pbr.frag.spv")
    .build();

// Setting up a render pipeline
let pipeline = RenderPipeline::builder()
    .with_vertex::<Vertex>()
    .with_material(material)
    .build();
```

### 3. Physics Integration

```rust
// Adding physics to an entity
let entity = Entity::builder()
    .with_component(RigidBody::new(1.0)) // mass
    .with_component(Collider::cube(1.0))  // size
    .build();

// Applying forces
if let Some(body) = scene.get_component_mut::<RigidBody>(entity) {
    body.apply_force(Vector3::new(0.0, 9.81, 0.0));
}
```

## Next Steps

1. Read the [Technical Reference](TechnicalReference.md) for detailed API documentation
2. Check out the [Directory Structure](DirectoryStructure.md) to understand the project layout
3. Review the [Contributing Guide](Contributing.md) if you want to help develop the engine

## Common Tasks

### Loading Assets

```rust
// Loading a model
let model = scene.resource_manager()
    .load::<Model>("models/character.gltf")?;

// Loading a texture
let texture = scene.resource_manager()
    .load::<Texture>("textures/albedo.png")?;
```

### Setting Up Input

```rust
window.on_input(|event| {
    match event {
        InputEvent::KeyPressed(Key::Space) => {
            // Handle jump
        }
        InputEvent::MouseMoved(x, y) => {
            // Handle camera movement
        }
        _ => {}
    }
});
```

## Troubleshooting

For common issues and solutions, see our [Technical Reference](TechnicalReference.md#troubleshooting).

Need more help? Check our full [Documentation Index](index.md).

---

<div align="center">

[Back to README](README.md) | [Technical Reference](TechnicalReference.md) | [Contributing](Contributing.md)

</div>
