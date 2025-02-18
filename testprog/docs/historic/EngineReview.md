# Engine Review

This document provides an overview of the current functionality of the graphics engine, along with suggestions for improvements and additions.

## Current Functionality

The engine is structured around several core modules:

- **Core:** This module contains the `Engine` struct, which is the central coordinator for all systems. It manages the main loop, window, graphics, resources, and scene. The `core` module also includes `Window`, `InputState`, and `Error` handling.
  - `Engine`: Initializes and manages the other core components (`Window`, `Graphics`, `ResourceManager`, `SceneManager`). Contains the main loop (`run`), update logic (`update`), and rendering logic (`render`).
  - `Window`: Uses the `winit` crate to create and manage the application window. Handles window events (resizing, moving, focusing, closing, mouse input). Provides methods to get window size, check for close requests, and access the raw window handle and input state.
  - `InputState`: Tracks mouse input (position, delta, button presses).
  - `Error`: Custom error type for the engine.
- **Graphics:** This module handles rendering using Vulkan. It includes components for managing the device (`DeviceContext`), swapchain (`SwapchainContext`), rendering pipeline (`RenderPipeline`), shaders (`ShaderModule`), and rendering context (`RenderContext`). It also defines vertex structures (`Vertex2D`, `Vertex3D`) and a basic `Mesh`.
- **Physics:** This module provides a custom physics engine with components for rigidbodies (`RigidBody`), colliders (`Collider`), and the physics world (`PhysicsWorld`). It supports collision detection and response.
- **Resource:** This module manages game assets. It includes an `AssetCache`, `ResourceManager`, and loaders for textures (`TextureLoader`), shaders (`ShaderLoader`), and models (`ModelLoader`). It defines `Asset` types for textures, shaders and models.
- **Scene:** This module implements an Entity-Component-System (ECS) architecture. It includes `Component`, `Entity`, `SceneManager`, `System`, and `World` definitions. This allows for organizing game objects and their behaviors in a modular way.
  - `Component`: Defines data components that can be attached to entities.
  - `Entity`: Represents a game object as a collection of components.
  - `SceneManager`: Manages the scene, including updating and rendering entities and systems.
  - `System`: Implements logic that operates on entities with specific components.
  - `World`: Contains all entities and components.
- **UI:** This module provides a basic UI framework with `Element`, `Layout`, `Style`, `Widget`, and `UIContext`.

The engine's main entry point is `testprog/main.rs`, which creates an `Engine` instance and runs the main loop. The engine uses `winit` for window management and Vulkan for graphics rendering.

_(Analysis in progress...)_

## Potential Improvements and Additions

1.  **Scene Graph:** Implement a scene graph for hierarchical transformations and object management.
2.  **Advanced Rendering:** Add support for shadows, PBR, and global illumination.
3.  **CAD-Specific Features:**
    - Precise input handling.
    - Constraint system.
    - Solid modeling operations (boolean, extrusion, revolution).
    - Parametric Modeling
4.  **Game Engine-Specific Features:**
    - Animation system.
    - Particle system.
    - AI and pathfinding.
    - Scripting.
5.  **Cross-Platform Compatibility:** Ensure the engine works on Windows, macOS, and Linux.
6.  **Improved UI:** Enhance the UI system with more complex layouts, widgets, and styling.
7.  **Resource Management:** Improve resource loading, unloading, and asynchronous operations.

## Implementation Strategies

1.  **Scene Graph:**
    - Modify `scene` module: Add `Node` struct (transformation, children, optional renderable object).
    - Implement hierarchical transformation calculations.
    - Modify rendering system to traverse scene graph.
2.  **Advanced Rendering:**
    - Research and implement in `graphics` module (new shaders, render passes, pipeline configurations).
3.  **CAD-Specific Features:**
    - **Precise Input:** Extend `InputState` in `core` module.
    - **Constraint System:** New `constraints` module.
    - **Solid Modeling:** New `modeling` module (likely using B-rep).
    - **Parametric Modeling:** Extend resource system and scene to handle parameters.
4.  **Game Engine-Specific Features:**
    - **Animation System:** New `animation` module.
    - **Particle System:** New `particles` module.
    - **AI and Pathfinding:** New `ai` and `pathfinding` modules.
    - **Scripting:** New `scripting` module, integrate a scripting language.
5.  **Cross-Platform Compatibility:** Test and debug on different platforms.
6.  **Improved UI:** Refactor `ui` module (consider retained-mode UI).
7.  **Resource Management:** Improve `resource` module (asynchronous loading, streaming, dependencies).
