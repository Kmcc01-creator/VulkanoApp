use std::collections::HashMap;
use vulkano::pipeline::graphics::vertex_input::{
    Vertex, VertexBufferDescription, VertexInputRate, VertexMemberInfo,
};

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex2D {
    pub position: [f32; 2],
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

unsafe impl Vertex for Vertex2D {
    fn per_vertex() -> VertexBufferDescription {
        let mut members = HashMap::new();
        members.insert(
            "position".to_string(),
            VertexMemberInfo {
                format: vulkano::format::Format::R32G32_SFLOAT,
                offset: 0,
                num_elements: 1,
            },
        );

        VertexBufferDescription {
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: VertexInputRate::Vertex,
            members,
        }
    }

    fn per_instance() -> VertexBufferDescription {
        VertexBufferDescription {
            stride: 0,
            input_rate: VertexInputRate::Instance { divisor: 1 },
            members: HashMap::new(),
        }
    }

    fn per_instance_with_divisor(divisor: u32) -> VertexBufferDescription {
        VertexBufferDescription {
            stride: 0,
            input_rate: VertexInputRate::Instance { divisor },
            members: HashMap::new(),
        }
    }
}

unsafe impl Vertex for MeshVertex {
    fn per_vertex() -> VertexBufferDescription {
        let mut members = HashMap::new();
        members.insert(
            "position".to_string(),
            VertexMemberInfo {
                format: vulkano::format::Format::R32G32B32_SFLOAT,
                offset: 0,
                num_elements: 1,
            },
        );
        members.insert(
            "normal".to_string(),
            VertexMemberInfo {
                format: vulkano::format::Format::R32G32B32_SFLOAT,
                offset: 12,
                num_elements: 1,
            },
        );
        members.insert(
            "uv".to_string(),
            VertexMemberInfo {
                format: vulkano::format::Format::R32G32_SFLOAT,
                offset: 24,
                num_elements: 1,
            },
        );

        VertexBufferDescription {
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: VertexInputRate::Vertex,
            members,
        }
    }

    fn per_instance() -> VertexBufferDescription {
        VertexBufferDescription {
            stride: 0,
            input_rate: VertexInputRate::Instance { divisor: 1 },
            members: HashMap::new(),
        }
    }

    fn per_instance_with_divisor(divisor: u32) -> VertexBufferDescription {
        VertexBufferDescription {
            stride: 0,
            input_rate: VertexInputRate::Instance { divisor },
            members: HashMap::new(),
        }
    }
}

unsafe impl Vertex for SpriteVertex {
    fn per_vertex() -> VertexBufferDescription {
        let mut members = HashMap::new();
        members.insert(
            "position".to_string(),
            VertexMemberInfo {
                format: vulkano::format::Format::R32G32_SFLOAT,
                offset: 0,
                num_elements: 1,
            },
        );
        members.insert(
            "uv".to_string(),
            VertexMemberInfo {
                format: vulkano::format::Format::R32G32_SFLOAT,
                offset: 8,
                num_elements: 1,
            },
        );

        VertexBufferDescription {
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: VertexInputRate::Vertex,
            members,
        }
    }

    fn per_instance() -> VertexBufferDescription {
        VertexBufferDescription {
            stride: 0,
            input_rate: VertexInputRate::Instance { divisor: 1 },
            members: HashMap::new(),
        }
    }

    fn per_instance_with_divisor(divisor: u32) -> VertexBufferDescription {
        VertexBufferDescription {
            stride: 0,
            input_rate: VertexInputRate::Instance { divisor },
            members: HashMap::new(),
        }
    }
}

unsafe impl Vertex for UiVertex {
    fn per_vertex() -> VertexBufferDescription {
        let mut members = HashMap::new();
        members.insert(
            "position".to_string(),
            VertexMemberInfo {
                format: vulkano::format::Format::R32G32_SFLOAT,
                offset: 0,
                num_elements: 1,
            },
        );
        members.insert(
            "uv".to_string(),
            VertexMemberInfo {
                format: vulkano::format::Format::R32G32_SFLOAT,
                offset: 8,
                num_elements: 1,
            },
        );
        members.insert(
            "color".to_string(),
            VertexMemberInfo {
                format: vulkano::format::Format::R32G32B32A32_SFLOAT,
                offset: 16,
                num_elements: 1,
            },
        );

        VertexBufferDescription {
            stride: std::mem::size_of::<Self>() as u32,
            input_rate: VertexInputRate::Vertex,
            members,
        }
    }

    fn per_instance() -> VertexBufferDescription {
        VertexBufferDescription {
            stride: 0,
            input_rate: VertexInputRate::Instance { divisor: 1 },
            members: HashMap::new(),
        }
    }

    fn per_instance_with_divisor(divisor: u32) -> VertexBufferDescription {
        VertexBufferDescription {
            stride: 0,
            input_rate: VertexInputRate::Instance { divisor },
            members: HashMap::new(),
        }
    }
}
