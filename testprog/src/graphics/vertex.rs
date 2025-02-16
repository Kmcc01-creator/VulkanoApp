use bytemuck::{Pod, Zeroable};
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

unsafe impl Vertex for Vertex3D {
    fn per_vertex() -> vulkano::pipeline::graphics::vertex_input::VertexInputState {
        use vulkano::pipeline::graphics::vertex_input::{
            VertexInputAttributeDescription, VertexInputBindingDescription,
        };

        // Single binding description for all vertex data
        let bindings = std::collections::HashMap::from([(
            0, // binding number
            VertexInputBindingDescription {
                stride: std::mem::size_of::<Self>() as u32,
                input_rate: vulkano::pipeline::graphics::vertex_input::VertexInputRate::Vertex,
            },
        )]);

        // Attribute descriptions for each vertex field
        let attributes = std::collections::HashMap::from([
            (
                0,
                VertexInputAttributeDescription {
                    format: vulkano::format::Format::R32G32B32_SFLOAT,
                    offset: memoffset::offset_of!(Self, position) as u32,
                },
            ),
            (
                1,
                VertexInputAttributeDescription {
                    format: vulkano::format::Format::R32G32B32_SFLOAT,
                    offset: memoffset::offset_of!(Self, normal) as u32,
                },
            ),
            (
                2,
                VertexInputAttributeDescription {
                    format: vulkano::format::Format::R32G32_SFLOAT,
                    offset: memoffset::offset_of!(Self, uv) as u32,
                },
            ),
            (
                3,
                VertexInputAttributeDescription {
                    format: vulkano::format::Format::R32G32B32A32_SFLOAT,
                    offset: memoffset::offset_of!(Self, color) as u32,
                },
            ),
        ]);

        vulkano::pipeline::graphics::vertex_input::VertexInputState {
            bindings,
            attributes,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

unsafe impl Vertex for Vertex2D {
    fn per_vertex() -> vulkano::pipeline::graphics::vertex_input::VertexInputState {
        use vulkano::pipeline::graphics::vertex_input::{
            VertexInputAttributeDescription, VertexInputBindingDescription,
        };

        let bindings = std::collections::HashMap::from([(
            0,
            VertexInputBindingDescription {
                stride: std::mem::size_of::<Self>() as u32,
                input_rate: vulkano::pipeline::graphics::vertex_input::VertexInputRate::Vertex,
            },
        )]);

        let attributes = std::collections::HashMap::from([
            (
                0,
                VertexInputAttributeDescription {
                    format: vulkano::format::Format::R32G32_SFLOAT,
                    offset: memoffset::offset_of!(Self, position) as u32,
                },
            ),
            (
                1,
                VertexInputAttributeDescription {
                    format: vulkano::format::Format::R32G32_SFLOAT,
                    offset: memoffset::offset_of!(Self, uv) as u32,
                },
            ),
            (
                2,
                VertexInputAttributeDescription {
                    format: vulkano::format::Format::R32G32B32A32_SFLOAT,
                    offset: memoffset::offset_of!(Self, color) as u32,
                },
            ),
        ]);

        vulkano::pipeline::graphics::vertex_input::VertexInputState {
            bindings,
            attributes,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex3D>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    pub fn create_quad() -> Self {
        let vertices = vec![
            Vertex3D {
                position: [-0.5, -0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex3D {
                position: [0.5, -0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex3D {
                position: [0.5, 0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex3D {
                position: [-0.5, 0.5, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        Self::new(vertices, indices)
    }
}
