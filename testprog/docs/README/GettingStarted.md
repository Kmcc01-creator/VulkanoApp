# Getting Started with the Engine 🚀

## Prerequisites

- Rust 2021 Edition or later
- Vulkan SDK 1.3+
- CMake 3.15+
- Git

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

## Project Layout

```
your-game/
├── src/
│   ├── main.rs       # Application entry point
│   ├── game.rs       # Game logic
│   ├── systems/      # Custom systems
│   └── components/   # Custom components
├── assets/
│   ├── models/       # 3D models
│   ├── textures/     # Texture files
│   └── shaders/      # Shader files
└── Cargo.toml        # Project configuration
```

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

### Creating UI

```rust
use engine::ui::{Button, Panel, Layout};

let ui = Panel::new()
    .with_layout(Layout::Vertical)
    .with_child(Button::new("Start Game"))
    .with_child(Button::new("Settings"))
    .with_child(Button::new("Quit"));
```

## Best Practices

1. **Resource Management**

   - Use the resource manager for asset loading
   - Clean up resources when no longer needed
   - Batch similar resources together

2. **Performance**

   - Use appropriate data structures
   - Batch similar operations
   - Profile your code regularly

3. **Error Handling**
   - Use Result for fallible operations
   - Provide meaningful error messages
   - Handle cleanup in error cases

## Next Steps

1. Check out the [Architecture Guide](../Architecture.md)
2. Review [Example Projects](../../examples/)
3. Join our [Discord Community](https://discord.gg/example)
4. Contribute to the project

## Troubleshooting

See our [Troubleshooting Guide](Troubleshooting.md) for common issues and solutions.

---

Need more help? Check our [Documentation](https://docs.example.com) or join our [Discord](https://discord.gg/example).
