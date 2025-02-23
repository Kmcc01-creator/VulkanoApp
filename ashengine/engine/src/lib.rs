//! AshEngine - A Vulkan-based graphics engine written in Rust

pub mod config;
pub mod error;
pub mod graphics;
pub mod helpers;
pub mod lighting;
pub mod memory;
pub mod mesh;
pub mod physics;
pub mod resource;
pub mod text;

// Re-exports for convenience
pub use error::{Result, VulkanError};
pub use graphics::{Pipeline, RenderPass, Renderer, Swapchain};

// Re-export all the types needed for text rendering
pub use text::{
    ndc_to_pixel, pixel_to_ndc, FontAtlas, TextAlignment, TextConfig, TextElement, TextLayout,
    TextPicker,
};
