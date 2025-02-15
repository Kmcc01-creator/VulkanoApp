use bytemuck::{Pod, Zeroable};
use vulkano::{buffer::Subbuffer, pipeline::graphics::vertex_input::Vertex};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable, Vertex)]
pub struct Vertex3D {
    #[format(vulkano::format::Format::R32G32B32_SFLOAT)]
    pub position: [f32; 3],

    #[format(vulkano::format::Format::R32G32B32_SFLOAT)]
    pub normal: [f32; 3],

    #[format(vulkano::format::Format::R32G32_SFLOAT)]
    pub uv: [f32; 2],

    #[format(vulkano::format::Format::R32G32B32A32_SFLOAT)]
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable, Vertex)]
pub struct Vertex2D {
    #[format(vulkano::format::Format::R32G32_SFLOAT)]
    pub position: [f32; 2],

    #[format(vulkano::format::Format::R32G32_SFLOAT)]
    pub uv: [f32; 2],

    #[format(vulkano::format::Format::R32G32B32A32_SFLOAT)]
    pub color: [f32; 4],
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
