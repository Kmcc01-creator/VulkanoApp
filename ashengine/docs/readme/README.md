# AshEngine Documentation

AshEngine is a Vulkan-based graphics engine written in Rust. This document provides an overview of the engine, its features, and how to use it.

## Table of Contents

- [Overview](#overview)
- [Project Structure](#project-structure)
- [Getting Started](#getting-started)
- [Core Concepts](#core-concepts)
  - [Context](#context)
  - [Renderer](#renderer)
  - [Swapchain](#swapchain)
  - [Commands](#commands)
  - [Pipelines](#pipelines)
  - [Shaders](#shaders)
  - [Meshes](#meshes)
  - [Memory Management](#memory-management)
- [Text Rendering](#text-rendering)
- [Configuration](#configuration)
- [Examples](#examples)

## Overview

AshEngine aims to provide a flexible and efficient foundation for building graphics applications and games using Vulkan. It offers features such as:

- Vulkan rendering pipeline management.
- Swapchain handling.
- Shader loading and management.
- Mesh loading and rendering.
- Text rendering.
- Resource management.
- Configuration system.

## Project Structure

The project is organized into the following main directories:

- `engine/`: Contains the core engine code.
  - `src/`: Source code for the engine.
    - `commands.rs`: Command buffer management.
    - `context.rs`: Vulkan context initialization and management.
    - `helpers.rs`: Utility functions.
    - `lib.rs`: Core library functions.
    - `main.rs`: Main entry point (likely for examples).
    - `mesh.rs`: Mesh data structures and functions.
    - `pipeline.rs`: Graphics pipeline management.
    - `renderer.rs`: Main rendering loop.
    - `resource.rs`: Resource management.
    - `shader.rs`: Shader loading and management.
    - `swapchain.rs`: Swapchain management.
    - `text/`: Text rendering functionality.
    - `config/`: Configuration system.
    - `memory/`: Memory management.
- `examples/`: Example applications demonstrating engine usage.
- `docs/`: Documentation (this directory).

## Getting Started

[Getting Started](./getting_started.md)

## Core Concepts

### Context

[Context](./context.md)

### Renderer

[Renderer](./renderer.md)

### Swapchain

[Swapchain](./swapchain.md)

### Commands

[Commands](./commands.md)

### Pipelines

[Pipelines](./pipelines.md)

### Shaders

[Shaders](./shaders.md)

### Meshes

[Meshes](./meshes.md)

### Memory Management

[Memory Management](./memory_management.md)

## Text Rendering

[Text Rendering](./text_rendering.md)

## Configuration

[Configuration](./configuration.md)

## Examples

[Examples](./examples.md)
