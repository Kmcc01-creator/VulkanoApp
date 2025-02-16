use std::collections::HashMap;
use vulkano::buffer::BufferContents;
use vulkano::format::Format;
use vulkano::pipeline::graphics::vertex_input::{
    Vertex, VertexBufferDescription, VertexInputRate, VertexMemberInfo,
};

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex2D {
    pub x: f32,
    pub y: f32,
}

unsafe impl Vertex for Vertex2D {
    fn per_vertex() -> VertexBufferDescription {
        VertexBufferDescription {
            stride: std::mem::size_of::<Self>() as u32,
            members: HashMap::from([(
                "position".to_string(),
                VertexMemberInfo {
                    offset: 0,
                    format: Format::R32G32_SFLOAT,
                    num_elements: 1,
                },
            )]),
            input_rate: VertexInputRate::Vertex,
        }
    }

    fn per_instance() -> VertexBufferDescription {
        Self::per_vertex()
    }

    fn per_instance_with_divisor(divisor: u32) -> VertexBufferDescription {
        let mut desc = Self::per_vertex();
        desc.input_rate = VertexInputRate::Instance { divisor };
        desc
    }
}
