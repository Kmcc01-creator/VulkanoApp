use glam::{Vec2, Vec3, Vec4};
use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[derive(Debug, Clone, Copy, BufferContents, Vertex)]
#[repr(C)]
pub struct UiVertex {
    #[location(0)]
    position: [f32; 2],
    #[location(1)]
    uv: [f32; 2],
    #[location(2)]
    color: [f32; 4],
}

impl UiVertex {
    pub fn new(position: Vec2, uv: Vec2, color: Vec4) -> Self {
        Self {
            position: position.to_array(),
            uv: uv.to_array(),
            color: color.to_array(),
        }
    }
}

#[derive(Debug, Clone, Copy, BufferContents, Vertex)]
#[repr(C)]
pub struct MeshVertex {
    #[location(0)]
    position: [f32; 3],
    #[location(1)]
    normal: [f32; 3],
    #[location(2)]
    uv: [f32; 2],
}

impl MeshVertex {
    pub fn new(position: Vec3, normal: Vec3, uv: Vec2) -> Self {
        Self {
            position: position.to_array(),
            normal: normal.to_array(),
            uv: uv.to_array(),
        }
    }
}

#[derive(Debug, Clone, Copy, BufferContents, Vertex)]
#[repr(C)]
pub struct SpriteVertex {
    #[location(0)]
    position: [f32; 2],
    #[location(1)]
    uv: [f32; 2],
}

impl SpriteVertex {
    pub fn new(position: Vec2, uv: Vec2) -> Self {
        Self {
            position: position.to_array(),
            uv: uv.to_array(),
        }
    }
}

// Helper function to create a quad mesh for UI elements
pub fn create_ui_quad(rect: [f32; 4], color: [f32; 4]) -> (Vec<UiVertex>, Vec<u32>) {
    let vertices = vec![
        UiVertex::new(
            Vec2::new(rect[0], rect[1]),
            Vec2::new(0.0, 0.0),
            Vec4::from(color),
        ),
        UiVertex::new(
            Vec2::new(rect[2], rect[1]),
            Vec2::new(1.0, 0.0),
            Vec4::from(color),
        ),
        UiVertex::new(
            Vec2::new(rect[2], rect[3]),
            Vec2::new(1.0, 1.0),
            Vec4::from(color),
        ),
        UiVertex::new(
            Vec2::new(rect[0], rect[3]),
            Vec2::new(0.0, 1.0),
            Vec4::from(color),
        ),
    ];

    let indices = vec![0, 1, 2, 2, 3, 0];

    (vertices, indices)
}

// Helper function to create a textured quad for sprites
pub fn create_sprite_quad(rect: [f32; 4], uv: [f32; 4]) -> (Vec<SpriteVertex>, Vec<u32>) {
    let vertices = vec![
        SpriteVertex::new(Vec2::new(rect[0], rect[1]), Vec2::new(uv[0], uv[1])),
        SpriteVertex::new(Vec2::new(rect[2], rect[1]), Vec2::new(uv[2], uv[1])),
        SpriteVertex::new(Vec2::new(rect[2], rect[3]), Vec2::new(uv[2], uv[3])),
        SpriteVertex::new(Vec2::new(rect[0], rect[3]), Vec2::new(uv[0], uv[3])),
    ];

    let indices = vec![0, 1, 2, 2, 3, 0];

    (vertices, indices)
}
