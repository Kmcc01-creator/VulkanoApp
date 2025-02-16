# Required Fixes for UI System Integration

## Modules Requiring Updates

1. Core Module:

   - `src/core/engine.rs`
   - `src/core/window.rs`

2. Graphics Module:

   - `src/graphics/shader.rs`
   - `src/graphics/pipeline.rs`
   - `src/graphics/renderer.rs`
   - `src/graphics/vertex.rs`
   - `src/graphics/mod.rs`

3. Physics Module:

   - `src/physics/rigidbody.rs`
   - `src/physics/physics_world.rs`

4. Resource Module:

   - `src/resource/mod.rs`
   - `src/resource/cache.rs`

5. Scene Module:
   - `src/scene/entity.rs`
   - `src/scene/mod.rs`

## Priority Fixes

### 1. Graphics Integration

```rust
// graphics/renderer.rs
impl RenderContext {
    pub fn new(graphics_queue: Arc<Queue>, swapchain: SwapchainContext) -> Result<Self, Error> {
        let graphics_queue = graphics_queue.clone(); // Fix move issue
        // ... rest of implementation
    }
}

// graphics/vertex.rs
impl Vertex for UiVertex {
    fn per_vertex() -> VertexBufferDescription {
        // Update to new Vulkano vertex format
    }
}
```

### 2. Physics System

```rust
// physics/rigidbody.rs
#[derive(Debug, Clone, PartialEq)]
pub enum BodyType {
    Static,
    Dynamic,
    Kinematic,
}
```

### 3. Resource Management

```rust
// resource/mod.rs
pub fn load<T: Asset>(&mut self, path: &Path) -> Result<Arc<T>, Error> {
    // Convert Box to Arc
    Ok(Arc::new(asset))
}
```

## Required Dependencies

```toml
[dependencies]
vulkano = "0.33"
glam = "0.24"
```

## Implementation Steps

1. Fix compile errors in order:

   - Core system dependencies
   - Graphics system integration
   - Resource management
   - Physics system
   - Scene system

2. Update UI integration:

   - Implement proper rendering pipeline
   - Add UI resource management
   - Integrate with physics for world space UI

3. Test and validate:
   - Verify basic UI rendering
   - Test UI interactions
   - Validate physics integration
