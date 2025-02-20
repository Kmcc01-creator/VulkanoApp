# AshEngine Documentation

## Overview

AshEngine is a modern Vulkan-based graphics engine written in Rust, designed to provide high-performance rendering capabilities with a clean and safe API.

## Recent Improvements

- **Window Management**: Implemented robust window resizing support
  - Proper swapchain recreation during window resize events
  - Automatic content scaling with window dimensions
  - Efficient handling of window maximize/minimize operations
  - Surface capabilities awareness for different display configurations

## Project Goals

- Create a robust, safe, and performant graphics engine using Vulkan
- Provide a clean, ergonomic API for graphics programming
- Maintain excellent documentation and examples
- Support modern rendering techniques and best practices
- Ensure proper resource management and safety through Rust's ownership system

## Future Development

- **Compute Shader Integration**: Planning to add compute shader support for:
  - Particle systems and physics simulations
  - Post-processing effects
  - General-purpose GPU computing (GPGPU) tasks
  - Real-time ray tracing acceleration
- **Advanced Rendering Features**:
  - Multiple render passes for advanced effects
  - Deferred rendering support
  - Dynamic lighting system
  - PBR material system
- **Performance Optimizations**:
  - Command buffer reuse strategies
  - Dynamic resource allocation
  - Async compute capabilities

## Technical Implementation

The engine is built on several core components:

- [Vulkan Context](./technical/VulkanContext.md) - Manages Vulkan instance, device, and surface creation
- [Render Pipeline](./technical/RenderPipeline.md) - Handles shader compilation and graphics pipeline setup
- [Memory Management](./technical/MemoryManagement.md) - Details buffer and image memory allocation
- [Command System](./technical/Commands.md) - Explains command buffer recording and submission
- [Synchronization](./technical/Synchronization.md) - Covers frame synchronization and resource access
- [AI Integration](./AIHandling.md) - Information about AI features and capabilities

## Getting Started

1. [Installation Guide](./guides/Installation.md)
2. [Basic Usage](./guides/BasicUsage.md)
3. [Creating Your First Scene](./guides/FirstScene.md)
4. [Advanced Features](./guides/AdvancedFeatures.md)

## How-To Guides

- [Setting Up a Project](./howto/ProjectSetup.md)
- [Working with Shaders](./howto/Shaders.md)
- [Creating Materials](./howto/Materials.md)
- [Scene Management](./howto/SceneManagement.md)
- [Performance Optimization](./howto/Performance.md)

## API Reference

- [Core API](./api/Core.md)
- [Graphics Pipeline](./api/Pipeline.md)
- [Resource Management](./api/Resources.md)
- [Utility Functions](./api/Utils.md)

## Examples

- [Basic Triangle](./examples/Triangle.md)
- [Textured Cube](./examples/TexturedCube.md)
- [Dynamic Lighting](./examples/DynamicLighting.md)
- [Multiple Renderpasses](./examples/MultipleRenderpasses.md)

## Contributing

- [Development Setup](./contributing/Setup.md)
- [Coding Standards](./contributing/Standards.md)
- [Pull Request Process](./contributing/PullRequests.md)
- [Documentation Guidelines](./contributing/Documentation.md)

## Troubleshooting

- [Common Issues](./troubleshooting/CommonIssues.md)
- [Debug Guide](./troubleshooting/Debugging.md)
- [Performance Problems](./troubleshooting/Performance.md)

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.
