# UI System Documentation

## Overview

The UI system provides a flexible and extensible framework for building both traditional 2D interfaces and in-game UI elements. It uses a component-based architecture with elements, layouts, and styles.

## Core Components

### Element (`element.rs`)

The fundamental building block of the UI system, providing:

- Hierarchical structure with parent-child relationships
- Event handling system (click, hover, key, and custom events)
- Widget integration for custom rendering
- Automatic layout computation
- Style application

Key features:

```rust
Element {
    id: String,
    children: Vec<Element>,
    layout: Layout,
    style: Style,
    computed_bounds: Rect,
    events: EventHandlers,
    widget: Option<Box<dyn Widget>>
}
```

### Layout (`layout.rs`)

Handles positioning and sizing of UI elements:

- Margin and padding control
- Minimum and maximum size constraints
- Content size calculation
- Automatic space distribution

### Style (`style.rs`)

Manages visual appearance:

- Background color
- Border color and width
- Corner radius for rounded elements
- Default theme values

## Event System

The UI system includes a robust event handling system supporting:

- Click events
- Hover states
- Keyboard input
- Custom event types
- Event bubbling through the element hierarchy

## Current Capabilities

1. **Element Management**

   - Dynamic element creation and modification
   - Parent-child relationships
   - Flexible layout system
   - Custom widget integration

2. **Visual Styling**

   - Color management
   - Border customization
   - Rounded corners
   - Default theme support

3. **Layout Control**

   - Margin and padding
   - Size constraints
   - Automatic space calculation
   - Nested element positioning

4. **Event Handling**
   - Mouse interaction
   - Keyboard input
   - Custom event types
   - Event propagation

## Planned Improvements

### Top-Layer UI

For traditional interface elements like file icons and system controls:

1. **Window Management System**

   - Draggable windows
   - Resizable containers
   - Z-index ordering
   - Window focus management

2. **Standard Widgets**

   - File icons
   - Buttons and inputs
   - Dropdown menus
   - Scrollable containers

3. **Theme System**
   - Global theme management
   - Theme inheritance
   - Dynamic theme switching
   - Custom theme creation

### In-Game UI

For elements that exist within the 3D game world:

1. **World Space Integration**

   - UI elements attached to 3D objects
   - Depth-aware rendering
   - Camera-based visibility
   - World space to screen space projection

2. **Interaction System**

   - Ray-traced element selection
   - Depth-based interaction priority
   - World space event handling
   - Object-specific UI behaviors

3. **Dynamic Positioning**

   - Object following
   - Screen space constraints
   - View frustum culling
   - Distance-based scaling

4. **Context-Aware UI**
   - Object state reflection
   - Interactive tooltips
   - Context menus
   - Status indicators

## Implementation Guidelines

1. **Top-Layer UI Implementation**

   - Build upon existing Element system
   - Implement window container widget
   - Add standard widget library
   - Create theme management system

2. **In-Game UI Implementation**

   - Extend Element with world space coordinates
   - Implement ray-tracing selection system
   - Add depth testing for UI elements
   - Create world space event system

3. **Integration Points**
   - Shared event system for both UI types
   - Common styling system
   - Unified widget interface
   - Consistent layout calculations

## Usage Example

```rust
// Create a UI element
let mut element = Element::new()
    .with_id("my-button")
    .with_style(Style::new()
        .with_background_color(Vec4::new(0.2, 0.2, 0.2, 1.0))
        .with_corner_radius(5.0))
    .with_layout(Layout::new()
        .with_padding(Vec2::new(10.0, 10.0)))
    .on_click(|| println!("Button clicked!"))
    .on_hover(|hover| println!("Hover state: {}", hover));
```
