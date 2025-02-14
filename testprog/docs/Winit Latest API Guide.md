# Winit 0.30.9 API Guide

## Window Creation

In winit 0.30.9, window creation should be done through the `EventLoopBuilder` and direct window creation:

```rust
use winit::event_loop::EventLoopBuilder;
use winit::window::Window;

// Create event loop
let event_loop = EventLoopBuilder::new().build()?;

// Create window
let window = Window::new(&event_loop)?;
```

## Key Components

### EventLoop Configuration

```rust
use winit::event_loop::EventLoopBuilder;

let event_loop = EventLoopBuilder::new()
    .with_any_thread(true)  // Optional: Allow running on any thread
    .build()?;
```

### Window Configuration

```rust
use winit::window::Window;
use winit::dpi::LogicalSize;

let window = Window::new(&event_loop)?;
window.set_title("Window Title");
window.set_inner_size(LogicalSize::new(800, 600));
```

### Event Handling

```rust
event_loop.run(move |event, window_target| {
    match event {
        Event::WindowEvent { event, .. } => {
            // Handle window events
        }
        Event::AboutToWait => {
            // Frame update logic
        }
        _ => (),
    }
})?;
```

## Important Changes from Previous Versions

1. `WindowBuilder` is no longer the recommended way to create windows
2. Event loop control is handled through `EventLoopWindowTarget`
3. `AboutToWait` replaces `MainEventsCleared`
4. Window attributes are set after creation

## Best Practices

### Window Creation

```rust
// DO this:
let window = Window::new(&event_loop)?;
window.set_title("Title");
window.set_inner_size(size);

// DON'T do this:
// let window = WindowBuilder::new()...  // Deprecated pattern
```

### Event Loop Control

```rust
// DO this:
window_target.exit();  // To exit the event loop

// DON'T do this:
// *control_flow = ControlFlow::Exit;  // Old pattern
```

## Common Operations

### Window Positioning

```rust
if let Some(monitor) = window.current_monitor() {
    let monitor_size = monitor.size();
    let window_size = window.outer_size();
    let x = (monitor_size.width - window_size.width) / 2;
    let y = (monitor_size.height - window_size.height) / 2;
    window.set_outer_position(PhysicalPosition::new(x, y));
}
```

### Input Handling

```rust
match event {
    WindowEvent::MouseInput { state, button, .. } => {
        // Handle mouse input
    }
    WindowEvent::CursorMoved { position, .. } => {
        // Handle cursor movement
    }
    // More event handling...
}
```

## Error Handling

Proper error handling with Results:

```rust
let window = Window::new(&event_loop)
    .map_err(|e| Error::WindowCreation(e.to_string()))?;
```

## Thread Safety

Event loop considerations:

```rust
// Main thread event loop
event_loop.run(move |event, window_target| {
    // Event handling
})?;

// For multi-threaded applications
let event_loop = EventLoopBuilder::new()
    .with_any_thread(true)
    .build()?;
```

## Tips and Tricks

1. Always handle the `AboutToWait` event for frame updates
2. Use `window_target.exit()` for clean shutdown
3. Implement proper error handling for window operations
4. Consider platform-specific features when needed

## Testing

Create a basic window test:

```rust
#[test]
fn test_window_creation() {
    let event_loop = EventLoopBuilder::new().build().unwrap();
    let window = Window::new(&event_loop).unwrap();
    assert!(window.is_visible());
}
```

## Further Reading

1. [Winit Documentation](https://docs.rs/winit/0.30.9/winit/)
2. [Window Creation Guide](https://docs.rs/winit/0.30.9/winit/window/struct.Window.html)
3. [Event Loop Documentation](https://docs.rs/winit/0.30.9/winit/event_loop/struct.EventLoop.html)
