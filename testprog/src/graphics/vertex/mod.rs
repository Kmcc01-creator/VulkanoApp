use std::collections::HashMap;
use vulkano::pipeline::graphics::vertex_input::{
    Vertex, VertexBufferDescription, VertexInputAttributeDescription,
    VertexInputBindingDescription, VertexInputRate,
};

pub mod prelude;
mod types;

pub use types::*;

/// Unified trait for vertex attribute descriptions and bindings
pub trait VertexLayout: Sized {
    fn binding_description() -> VertexInputBindingDescription;
    fn attribute_descriptions() -> Vec<VertexInputAttributeDescription>;
}

/// Builder for vertex buffer bindings
#[derive(Debug, Clone)]
pub struct VertexBindingDescription {
    pub binding: u32,
    pub stride: u32,
    pub input_rate: VertexInputRate,
}

/// Builder for vertex buffer attributes
#[derive(Debug, Clone)]
pub struct VertexAttributeDescription {
    pub binding: u32,
    pub location: u32,
    pub format: vulkano::format::Format,
    pub offset: u32,
}

impl VertexBindingDescription {
    pub fn new(binding: u32, stride: u32) -> Self {
        Self {
            binding,
            stride,
            input_rate: VertexInputRate::Vertex,
        }
    }

    pub fn input_rate(mut self, rate: VertexInputRate) -> Self {
        self.input_rate = rate;
        self
    }

    pub fn instance(self, divisor: u32) -> Self {
        self.input_rate(VertexInputRate::Instance { divisor })
    }

    pub fn build(self) -> VertexInputBindingDescription {
        VertexInputBindingDescription {
            binding: self.binding,
            stride: self.stride,
            input_rate: self.input_rate,
        }
    }
}

impl VertexAttributeDescription {
    pub fn new(location: u32, binding: u32) -> Self {
        Self {
            location,
            binding,
            format: vulkano::format::Format::R32G32_SFLOAT,
            offset: 0,
        }
    }

    pub fn format(mut self, format: vulkano::format::Format) -> Self {
        self.format = format;
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    pub fn build(self) -> VertexInputAttributeDescription {
        VertexInputAttributeDescription {
            binding: self.binding,
            location: self.location,
            format: self.format,
            offset: self.offset,
        }
    }
}

// Helper trait for interleaved vertex data
pub trait InterleavedVertex: Vertex + VertexLayout {
    fn vertex_stride() -> u32 {
        std::mem::size_of::<Self>() as u32
    }
}

// Re-export vertex types
pub use types::{ColoredVertex, InstanceData, TexturedVertex, Vertex2D, Vertex3D};
