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

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct SpriteVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct UiVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex2D {
    fn buffer_description() -> VertexBufferDescription {
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
}

unsafe impl Vertex for Vertex2D {
    fn per_vertex() -> VertexBufferDescription {
        Self::buffer_description()
    }

    fn per_instance() -> VertexBufferDescription {
        Self::buffer_description()
    }

    fn per_instance_with_divisor(divisor: u32) -> VertexBufferDescription {
        let mut desc = Self::buffer_description();
        desc.input_rate = VertexInputRate::Instance { divisor };
        desc
    }
}

impl MeshVertex {
    fn buffer_description() -> VertexBufferDescription {
        VertexBufferDescription {
            stride: std::mem::size_of::<Self>() as u32,
            members: HashMap::from([
                (
                    "position".to_string(),
                    VertexMemberInfo {
                        offset: 0,
                        format: Format::R32G32B32_SFLOAT,
                        num_elements: 1,
                    },
                ),
                (
                    "normal".to_string(),
                    VertexMemberInfo {
                        offset: 12,
                        format: Format::R32G32B32_SFLOAT,
                        num_elements: 1,
                    },
                ),
                (
                    "uv".to_string(),
                    VertexMemberInfo {
                        offset: 24,
                        format: Format::R32G32_SFLOAT,
                        num_elements: 1,
                    },
                ),
            ]),
            input_rate: VertexInputRate::Vertex,
        }
    }
}

unsafe impl Vertex for MeshVertex {
    fn per_vertex() -> VertexBufferDescription {
        Self::buffer_description()
    }

    fn per_instance() -> VertexBufferDescription {
        Self::buffer_description()
    }

    fn per_instance_with_divisor(divisor: u32) -> VertexBufferDescription {
        let mut desc = Self::buffer_description();
        desc.input_rate = VertexInputRate::Instance { divisor };
        desc
    }
}

impl SpriteVertex {
    fn buffer_description() -> VertexBufferDescription {
        VertexBufferDescription {
            stride: std::mem::size_of::<Self>() as u32,
            members: HashMap::from([
                (
                    "position".to_string(),
                    VertexMemberInfo {
                        offset: 0,
                        format: Format::R32G32_SFLOAT,
                        num_elements: 1,
                    },
                ),
                (
                    "uv".to_string(),
                    VertexMemberInfo {
                        offset: 8,
                        format: Format::R32G32_SFLOAT,
                        num_elements: 1,
                    },
                ),
            ]),
            input_rate: VertexInputRate::Vertex,
        }
    }
}

unsafe impl Vertex for SpriteVertex {
    fn per_vertex() -> VertexBufferDescription {
        Self::buffer_description()
    }

    fn per_instance() -> VertexBufferDescription {
        Self::buffer_description()
    }

    fn per_instance_with_divisor(divisor: u32) -> VertexBufferDescription {
        let mut desc = Self::buffer_description();
        desc.input_rate = VertexInputRate::Instance { divisor };
        desc
    }
}

impl UiVertex {
    fn buffer_description() -> VertexBufferDescription {
        VertexBufferDescription {
            stride: std::mem::size_of::<Self>() as u32,
            members: HashMap::from([
                (
                    "position".to_string(),
                    VertexMemberInfo {
                        offset: 0,
                        format: Format::R32G32_SFLOAT,
                        num_elements: 1,
                    },
                ),
                (
                    "uv".to_string(),
                    VertexMemberInfo {
                        offset: 8,
                        format: Format::R32G32_SFLOAT,
                        num_elements: 1,
                    },
                ),
                (
                    "color".to_string(),
                    VertexMemberInfo {
                        offset: 16,
                        format: Format::R32G32B32A32_SFLOAT,
                        num_elements: 1,
                    },
                ),
            ]),
            input_rate: VertexInputRate::Vertex,
        }
    }
}

unsafe impl Vertex for UiVertex {
    fn per_vertex() -> VertexBufferDescription {
        Self::buffer_description()
    }

    fn per_instance() -> VertexBufferDescription {
        Self::buffer_description()
    }

    fn per_instance_with_divisor(divisor: u32) -> VertexBufferDescription {
        let mut desc = Self::buffer_description();
        desc.input_rate = VertexInputRate::Instance { divisor };
        desc
    }
}
