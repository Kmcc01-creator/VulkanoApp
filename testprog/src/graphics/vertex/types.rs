use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexBufferDescription, VertexInputRate};

macro_rules! impl_vertex_type {
    ($type:ty, $($field:ident: $offset:expr => $format:expr),*) => {
        unsafe impl Vertex for $type {
            fn per_vertex() -> VertexBufferDescription {
                let mut members = HashMap::new();
                $(
                    members.insert(
                        stringify!($field).to_string(),
                        vulkano::pipeline::graphics::vertex_input::VertexMemberInfo {
                            format: $format,
                            offset: $offset,
                            num_elements: 1,
                        }
                    );
                )*

                VertexBufferDescription {
                    stride: std::mem::size_of::<Self>() as u32,
                    input_rate: VertexInputRate::Vertex,
                    members,
                }
            }

            fn per_instance() -> VertexBufferDescription {
                VertexBufferDescription {
                    stride: std::mem::size_of::<Self>() as u32,
                    input_rate: VertexInputRate::Instance { divisor: 1 },
                    members: HashMap::new(),
                }
            }

            fn per_instance_with_divisor(divisor: u32) -> VertexBufferDescription {
                VertexBufferDescription {
                    stride: std::mem::size_of::<Self>() as u32,
                    input_rate: VertexInputRate::Instance { divisor },
                    members: HashMap::new(),
                }
            }
        }
    };
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex2D {
    pub position: [f32; 2],
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct ColoredVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct TexturedVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    pub world_matrix: [[f32; 4]; 4],
    pub color: [f32; 4],
}

impl_vertex_type!(Vertex2D,
    position: 0 => vulkano::format::Format::R32G32_SFLOAT
);

impl_vertex_type!(Vertex3D,
    position: 0 => vulkano::format::Format::R32G32B32_SFLOAT,
    normal: 12 => vulkano::format::Format::R32G32B32_SFLOAT,
    uv: 24 => vulkano::format::Format::R32G32_SFLOAT
);

impl_vertex_type!(ColoredVertex,
    position: 0 => vulkano::format::Format::R32G32B32_SFLOAT,
    color: 12 => vulkano::format::Format::R32G32B32A32_SFLOAT
);

impl_vertex_type!(TexturedVertex,
    position: 0 => vulkano::format::Format::R32G32_SFLOAT,
    uv: 8 => vulkano::format::Format::R32G32_SFLOAT
);

impl_vertex_type!(InstanceData,
    world_matrix: 0 => vulkano::format::Format::R32G32B32A32_SFLOAT,
    color: 64 => vulkano::format::Format::R32G32B32A32_SFLOAT
);
