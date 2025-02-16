# UI System Implementation

## Current Implementation Status

The UI system has been implemented with both top-layer UI (like windows and buttons) and in-game UI support (UI elements that exist in world space). The system uses a mock renderer for testing and development purposes, which will later be replaced with a full Vulkan renderer.

### Core Components

#### Element System

- Hierarchical tree structure for UI elements
- Event propagation through the tree
- Layout calculation and style application
- Support for both screen-space and world-space elements

```rust
let element = Element::new()
    .with_id("my_button")
    .with_widget(Button::new("Click me"))
    .with_style(Style::new().with_padding(Padding::uniform(5.0)))
    .on_click(|| println!("Clicked!"));
```

#### Layout System

- Flexible positioning (Static, Relative, Absolute)
- Margin and padding support
- Size constraints (min/max)
- Automatic content area calculation

```rust
let layout = Layout::new()
    .with_margin(Vec2::new(10.0, 10.0))
    .with_padding(Vec2::new(5.0, 5.0))
    .with_min_size(Vec2::new(100.0, 50.0))
    .with_position(PositionType::Absolute(Vec2::new(20.0, 20.0)));
```

#### Style System

- Background color
- Border color and width
- Corner radius
- Text color
- Title bar color (for windows)
- Padding configuration

```rust
let style = Style::new()
    .with_background_color(Vec4::new(0.2, 0.2, 0.2, 0.9))
    .with_border_color(Vec4::new(0.3, 0.3, 0.3, 1.0))
    .with_text_color(Vec4::new(1.0, 1.0, 1.0, 1.0))
    .with_corner_radius(5.0);
```

#### Widget System

1. Button

   - Clickable text button with hover state
   - Configurable size and appearance
   - Custom click handlers

2. Window

   - Draggable window container
   - Title bar with custom text
   - Configurable size and appearance
   - Child element container

3. WorldSpaceWidget
   - Wrapper for displaying any widget in world space
   - Automatic screen-space projection
   - Depth-aware rendering (planned)
   - World position tracking

### Mock Renderer

The current implementation uses a mock renderer for testing and development:

```rust
pub struct MockRenderer {
    viewport_size: Vec2,
}

// Implements drawing methods:
- draw_rect()
- draw_rect_outline()
- draw_text()
- measure_text()
- world_to_screen()
```

This allows testing UI layouts and interactions before implementing the full Vulkan renderer.

### Event System

- Click events
- Hover events
- Key events
- Custom events
- Event bubbling through the element tree

## Example Usage

### Top-Layer UI (Windows and Buttons)

```rust
use testprog::ui::prelude::*;

let mut window = Element::new()
    .with_id("main_window")
    .with_widget(Window::new("Controls"))
    .with_style(
        Style::new()
            .with_background_color(Vec4::new(0.2, 0.2, 0.2, 0.9))
            .with_border_color(Vec4::new(0.3, 0.3, 0.3, 1.0))
    )
    .with_layout(
        Layout::new()
            .with_position(PositionType::Absolute(Vec2::new(10.0, 10.0)))
    );

window.add_child(
    Element::new()
        .with_widget(Button::new("Click me"))
        .with_style(Style::new().with_padding(Padding::uniform(5.0)))
        .on_click(|| println!("Button clicked!"))
);
```

### In-Game UI (World Space Elements)

```rust
let world_ui = Element::new()
    .with_id("object_label")
    .with_widget(
        WorldSpaceWidget::new(
            Button::new("3D Object"),
            Vec3::new(0.0, 1.0, 0.0), // World position
            Vec2::new(100.0, 30.0)    // Size in screen space
        )
    )
    .with_style(
        Style::new()
            .with_background_color(Vec4::new(0.2, 0.2, 0.2, 0.8))
            .with_text_color(Vec4::new(1.0, 1.0, 1.0, 1.0))
    )
    .on_hover(|hover| println!("Object hover: {}", hover));
```

## Next Steps

1. Vulkan Renderer Integration

   - Implement shader system for UI rendering
   - Create vertex buffers for UI elements
   - Add texture support for icons and images
   - Implement proper text rendering

2. Advanced Features

   - Add scrollable containers
   - Implement drag and drop
   - Add UI animations
   - Implement focus system
   - Add clipboard support

3. Performance Optimizations

   - Batch similar draw calls
   - Implement UI element pooling
   - Add dirty flagging for layout recalculation
   - Optimize event propagation

4. Additional Widgets
   - Text input fields
   - Checkboxes
   - Radio buttons
   - Dropdown menus
   - Scrollbars
   - Progress bars
