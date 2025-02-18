// Re-export vertex types
pub use super::types::{ColoredVertex, InstanceData, TexturedVertex, Vertex2D, Vertex3D};

// Re-export vertex traits and builders
pub use super::{
    InterleavedVertex, VertexAttributeDescription, VertexBindingDescription, VertexLayout,
};

// Re-export Vulkano traits and types
pub use vulkano::pipeline::graphics::vertex_input::{
    Vertex, VertexBufferDescription, VertexInputAttributeDescription,
    VertexInputBindingDescription, VertexInputRate,
};

// Common vertex formats
pub use vulkano::format::Format;

pub const POSITION2_FORMAT: Format = Format::R32G32_SFLOAT;
pub const POSITION3_FORMAT: Format = Format::R32G32B32_SFLOAT;
pub const NORMAL3_FORMAT: Format = Format::R32G32B32_SFLOAT;
pub const UV2_FORMAT: Format = Format::R32G32_SFLOAT;
pub const COLOR4_FORMAT: Format = Format::R32G32B32A32_SFLOAT;

// Helper functions
pub fn vertex_binding(binding: u32, stride: u32) -> VertexBindingDescription {
    VertexBindingDescription::new(binding, stride)
}

pub fn instance_binding(binding: u32, stride: u32, divisor: u32) -> VertexBindingDescription {
    VertexBindingDescription::new(binding, stride).instance(divisor)
}

pub fn vertex_attribute(location: u32, binding: u32) -> VertexAttributeDescription {
    VertexAttributeDescription::new(location, binding)
}
