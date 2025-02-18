# Github References and API Patterns

## Winit 0.30.9

- Repository: https://github.com/rust-windowing/winit
- Documentation: https://docs.rs/winit/0.30.9/winit/

### Key Changes and Patterns

1. **Window Creation**

   ```rust
   // Modern approach (0.30.9)
   let event_loop = EventLoopBuilder::new().build()?;
   let window_attributes = winit::window::WindowAttributes {
       title: String::from("Title"),
       inner_size: Some(LogicalSize::new(800, 600).into()),
       ..Default::default()
   };
   let window = Window::create(&event_loop, window_attributes)?;
   ```

2. **Event Loop**
   ```rust
   event_loop.run(move |event, window_target| {
       match event {
           Event::AboutToWait => {
               // Frame update logic
               window_target.exit();
           }
           // Other event handling
       }
   });
   ```

## Vulkano 0.35.1

- Repository: https://github.com/vulkano-rs/vulkano
- Documentation: https://docs.rs/vulkano/0.35.1/vulkano/

### Key Patterns

1. **Instance Creation**

   ```rust
   let instance = Instance::new(InstanceCreateInfo {
       enabled_extensions: vulkano_win::required_extensions(),
       ..Default::default()
   })?;
   ```

2. **Device Selection**

   ```rust
   let device_extensions = DeviceExtensions {
       khr_swapchain: true,
       ..DeviceExtensions::empty()
   };

   let physical = instance.enumerate_physical_devices()?.next().unwrap();
   let queue_family_index = physical.queue_family_properties()
       .iter()
       .enumerate()
       .find(|(_, q)| q.queue_flags.graphics)
       .map(|(i, _)| i as u32)
       .unwrap();
   ```

3. **Pipeline Creation**
   ```rust
   let pipeline = GraphicsPipeline::start()
       .render_pass(subpass)
       .vertex_input_state(Vertex::per_vertex())
       .vertex_shader(vs.entry_point("main").unwrap(), ())
       .fragment_shader(fs.entry_point("main").unwrap(), ())
       .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
       .build(device.clone())?;
   ```

## Vulkano-Shaders 0.35

- Repository: https://github.com/vulkano-rs/vulkano/tree/master/vulkano-shaders
- Documentation: https://docs.rs/vulkano-shaders/0.35.0/vulkano_shaders/

### Key Patterns

1. **Shader Compilation**

   ```rust
   mod vs {
       vulkano_shaders::shader! {
           ty: "vertex",
           src: "
               #version 450
               layout(location = 0) in vec2 position;
               void main() {
                   gl_Position = vec4(position, 0.0, 1.0);
               }
           "
       }
   }
   ```

2. **Loading SPIR-V**
   ```rust
   let shader = unsafe {
       ShaderModule::from_bytes(device.clone(), spv_bytes)
   }?;
   ```

## Vulkano-Util 0.35

- Repository: https://github.com/vulkano-rs/vulkano/tree/master/vulkano-util
- Documentation: https://docs.rs/vulkano-util/0.35.0/vulkano_util/

### Key Patterns

1. **Context Creation**
   ```rust
   let context = VulkanoContext::new(VulkanoConfig::default())?;
   let renderer = VulkanoWindowRenderer::new(
       context,
       window_handle,
       RenderConfig::default(),
   )?;
   ```

## Bytemuck 1.9

- Repository: https://github.com/Lokathor/bytemuck
- Documentation: https://docs.rs/bytemuck/1.9.0/bytemuck/

### Key Patterns

1. **Pod and Zeroable Implementation**

   ```rust
   #[derive(Clone, Copy, Pod, Zeroable)]
   #[repr(C)]
   struct Vertex {
       position: [f32; 2],
       color: [f32; 3],
   }
   ```

2. **Casting**
   ```rust
   let vertices_bytes = bytemuck::cast_slice(&vertices);
   ```

## Breaking Changes Summary

### Winit 0.30.9

- Removed WindowBuilder in favor of direct Window creation
- Changed event loop control flow mechanism
- Updated window attribute handling
- Changed event types and patterns

### Vulkano 0.35.1

- Updated pipeline creation syntax
- Changed shader module handling
- Updated memory allocation patterns
- Modified swapchain creation

### Vulkano-Shaders 0.35

- Updated shader compilation macros
- Changed SPIR-V loading patterns
- Modified shader reflection API

### Vulkano-Util 0.35

- Updated context creation patterns
- Modified renderer initialization
- Changed configuration handling

### Bytemuck 1.9

- Added new derive macros
- Updated casting functions
- Modified alignment checking

## Best Practices

1. **Window Management**

   - Use EventLoopBuilder for window creation
   - Handle window events through the new event system
   - Properly manage window state

2. **Vulkan Integration**

   - Use proper synchronization primitives
   - Handle device memory appropriately
   - Implement proper cleanup

3. **Shader Management**

   - Use the shader! macro for compile-time verification
   - Properly handle shader resources
   - Implement proper error handling

4. **Memory Safety**
   - Use bytemuck for safe type casting
   - Implement proper alignment
   - Handle buffer creation safely

## Repository Links

- [Winit Issues](https://github.com/rust-windowing/winit/issues)
- [Vulkano Discussions](https://github.com/vulkano-rs/vulkano/discussions)
- [Bytemuck Examples](https://github.com/Lokathor/bytemuck/tree/master/examples)
