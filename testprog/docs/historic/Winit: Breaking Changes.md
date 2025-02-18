# Winit Breaking Changes (0.28 - 0.30.9)

## Major Architectural Changes

### 1. Event Loop Changes

#### Before (0.28 and earlier):

```rust
let event_loop = EventLoop::new();
event_loop.run(|event, _, control_flow| {
    *control_flow = ControlFlow::Poll;
    // Event handling
});
```

#### After (0.29+):

```rust
let event_loop = EventLoopBuilder::new().build()?;
event_loop.run(|event, window_target| {
    // Event handling
    window_target.exit(); // Instead of control_flow
});
```

### 2. Window Creation

#### Before (0.28):

```rust
let window = WindowBuilder::new()
    .with_title("Window")
    .with_inner_size(LogicalSize::new(800, 600))
    .build(&event_loop)
    .unwrap();
```

#### After (0.29+):

```rust
let window = Window::new(&event_loop)?;
window.set_title("Window");
window.set_inner_size(LogicalSize::new(800, 600));
```

## Key Breaking Changes

1. **Event Loop Control**

   - Removed `ControlFlow` enum
   - Introduced `WindowTarget` control methods
   - `run_return` method removed
   - New error handling with `Result` returns

2. **Window Management**

   - `WindowBuilder` deprecated in favor of direct window creation
   - Window attributes set after creation instead of during building
   - More explicit error handling with `Result` returns

3. **Event Handling**

   - Event types reorganized
   - `MainEventsCleared` replaced with `AboutToWait`
   - New event filtering capabilities
   - Event loop closure signature changed

4. **State Management**
   - Event loop state must be managed differently
   - Window target provides more control over application lifecycle
   - More explicit state synchronization required

## Migration Guide

### Event Loop Creation

```rust
// Old
let event_loop = EventLoop::new();

// New
let event_loop = EventLoop::new()?;
```

### Window Creation

```rust
// Old
let window = WindowBuilder::new()
    .with_title("Title")
    .with_inner_size(size)
    .build(&event_loop)?;

// New
let window = Window::new(&event_loop)?;
window.set_title("Title");
window.set_inner_size(size);
```

### Event Handling

```rust
// Old
*control_flow = ControlFlow::Poll;
if should_exit {
    *control_flow = ControlFlow::Exit;
}

// New
if should_exit {
    window_target.exit();
}
```

### State Management

```rust
// Old
Event::MainEventsCleared => {
    window.request_redraw();
}

// New
Event::AboutToWait => {
    window.request_redraw();
}
```

## Best Practices

1. **Error Handling**

   ```rust
   // Proper error handling for window creation
   let window = Window::new(&event_loop)
       .map_err(|e| Error::WindowCreation(e.to_string()))?;
   ```

2. **Event Loop Management**

   ```rust
   // Use structured event handling
   match event {
       Event::WindowEvent { event, .. } => {
           // Handle window events
       }
       Event::AboutToWait => {
           // Update application state
       }
       _ => (),
   }
   ```

3. **Window State Updates**
   ```rust
   // Handle window state changes explicitly
   WindowEvent::Resized(size) => {
       window.handle_resize(size);
   }
   ```

## Common Pitfalls

1. Trying to use deprecated `WindowBuilder` patterns
2. Using removed `ControlFlow` variants
3. Incorrect event loop closure signatures
4. Missing error handling for window operations
5. Using removed event types

## Future Considerations

- Event loop API may continue to evolve
- More explicit error handling likely to be added
- Further simplification of window management
- Enhanced state management capabilities

## Resources

- [Winit Repository](https://github.com/rust-windowing/winit)
- [API Documentation](https://docs.rs/winit/latest/winit/)
- [Migration Guide](https://github.com/rust-windowing/winit/blob/master/CHANGELOG.md)
