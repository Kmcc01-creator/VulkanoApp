// Re-export vertex types
pub use super::types::{ColoredVertex, InstanceData, TexturedVertex, Vertex2D, Vertex3D};
pub use super::MeshVertex;

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
pub fn vertex_binding<T: Vertex>(binding: u32) -> VertexInputBindingDescription {
    T::per_vertex()
        .binding_descriptions()
        .into_iter()
        .next()
        .unwrap()
}

pub fn instance_binding<T: Vertex>(binding: u32, divisor: u32) -> VertexInputBindingDescription {
    let mut desc = T::per_vertex()
        .binding_descriptions()
        .into_iter()
        .next()
        .unwrap();
    desc.input_rate = VertexInputRate::Instance { divisor };
    desc
}

pub fn vertex_attribute<T: Vertex>(location: u32, binding: u32) -> VertexInputAttributeDescription {
    T::per_vertex()
        .attribute_descriptions()
        .into_iter()
        .next()
        .unwrap()
}
