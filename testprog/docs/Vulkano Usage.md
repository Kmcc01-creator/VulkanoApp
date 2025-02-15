# Vulkano Usage: Best Practices

This document outlines best practices for using the Vulkano library. It's based on the library's design philosophy, the structure of the provided examples, and general Vulkan/Rust best practices.

## Core Principles

Vulkano is designed with the following principles in mind:

- **Safety:** Preventing invalid Vulkan API usage through compile-time and runtime checks. Unsafe code should be minimized and carefully reviewed.
- **Synchronization:** Automatic handling of GPU-side synchronization to simplify development and reduce errors. Customization is possible through unsafe trait implementations.
- **Convenience:** Striving for a user-friendly API that doesn't require excessive boilerplate or deep Vulkan expertise.
- **Completeness:** Aiming to cover all possible Vulkan usages, not just the most common ones.

## Getting Started

1.  **Setup:** Follow the platform-specific setup instructions in the Vulkano README (building requires CMake, Python, and potentially Ninja). You generally _do not_ need the full Vulkan SDK, except on macOS/iOS/tvOS where MoltenVK is required.
2.  **Dependencies:** Add `vulkano` to your `Cargo.toml`. If you're using the `shader!` macro, you'll also need `vulkano-shaders`. Consider using `vulkano-util` for common tasks like window and swapchain creation.
    ```toml
    [dependencies]
    vulkano = "..." // Use the latest version
    vulkano-shaders = "..."
    vulkano-util = "..." // Optional, but recommended
    ```
3.  **Examples:** The `vulkano/examples/` directory is your primary resource for learning how to use Vulkano. Explore the examples to understand how different features are implemented. Start with simpler examples (like `triangle`) and gradually move to more complex ones.
4.  **Documentation:** Refer to the official Vulkano documentation on [docs.rs](https://docs.rs/vulkano) for detailed API information. The guide on [vulkano.rs](https://vulkano.rs/) is a good starting point, although it may be slightly outdated.

## Best Practices

- **Embrace the Builder Pattern:** Vulkano heavily uses the builder pattern for creating objects. This allows for flexible configuration and avoids long parameter lists. Get comfortable with chaining method calls.

- **Understand Memory Management:** Vulkan gives you fine-grained control over memory allocation. Vulkano provides memory allocators (like `StandardMemoryAllocator`) to simplify this. Choose the appropriate `MemoryUsage` for your buffers and images (e.g., `Upload` for frequently updated data, `DeviceLocal` for GPU-only access).

- **Use `vulkano-util`:** The `vulkano-util` crate provides helpful utilities for common tasks, such as creating a window, swapchain, and render pass. This can significantly reduce boilerplate code.

- **Leverage `shader!` Macro:** The `vulkano-shaders` crate provides the `shader!` macro for compiling GLSL shaders into Rust code. This provides compile-time checks and type safety. It also handles shader reflection, automatically generating Rust types for shader inputs and outputs.

- **Automatic Synchronization:** Vulkano's automatic synchronization is a powerful feature. Understand how it works and how to customize it if necessary. In most cases, you can rely on Vulkano to handle dependencies between command buffers and submissions.

- **Error Handling:** Vulkano uses `Result` extensively. Handle errors gracefully. Don't just `unwrap()` everywhere. Use proper error handling techniques (e.g., `match`, `?`, or dedicated error handling crates).

- **Command Buffers:** Understand the lifecycle of command buffers. Use secondary command buffers for reusable rendering operations.

- **Pipelines:** Graphics pipelines are central to Vulkan. Understand the different pipeline stages (vertex, fragment, compute, etc.) and how to configure them. Use pipeline layouts to define the interface between your shaders and your application.

- **Descriptors:** Descriptors connect resources (buffers, images, samplers) to your shaders. Use descriptor sets and descriptor set layouts to manage these connections. Consider using push descriptors for frequently updated data.

- **Render Passes:** Render passes define how rendering is performed. Understand the concept of subpasses and attachments. Use render passes to optimize rendering, especially on tile-based GPUs.

- **Keep up-to-date:** Vulkano is under active development. Check the changelog and update your dependencies regularly.

- **Use the examples:** The examples are the best way to learn how to use the library. They cover a wide range of use cases and are well-documented.

## Example Directory Structure (`vulkano/examples/`)

The `examples/` directory contains a wealth of information. Here's a brief overview of some key examples:

- **`triangle/`:** A basic example showing how to render a simple triangle. A good starting point.
- **`basic-compute-shader/`:** Demonstrates compute shader usage.
- **`deferred/`:** Illustrates deferred rendering techniques.
- **`instancing/`:** Shows how to use instancing to render multiple objects efficiently.
- **`ray-tracing/` and `ray-tracing-auto/`:** Examples of ray tracing using Vulkano.
- **`shader-types-*`:** Examples related to shader type generation and sharing.
- **`teapot/`:** A more complex example rendering a 3D model.

This document provides a high-level overview of Vulkano best practices. The best way to learn is to dive into the examples and experiment with the library.
