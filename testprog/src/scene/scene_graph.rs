use glam::{Mat4, Quat, Vec3};
use std::sync::Arc;

// Placeholder for now.  Will need to integrate with the graphics system.
pub struct RenderObject {
    // Add fields like mesh, material, etc.
    pub name: String,
}

pub struct Node {
    pub name: String,
    pub transform: Transform,
    pub children: Vec<Node>,
    pub render_object: Option<Arc<RenderObject>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Node {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transform: Transform::default(),
            children: Vec::new(),
            render_object: None,
        }
    }

    pub fn add_child(&mut self, child: Node) {
        self.children.push(child);
    }

    pub fn remove_child(&mut self, index: usize) -> Option<Node> {
        if index < self.children.len() {
            Some(self.children.remove(index))
        } else {
            None
        }
    }
    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    pub fn global_transform(&self, parent_transform: &Mat4) -> Mat4 {
        let local_transform = Mat4::from_scale_rotation_translation(
            self.transform.scale,
            self.transform.rotation,
            self.transform.position,
        );
        *parent_transform * local_transform
    }
    pub fn set_render_object(&mut self, render_object: Arc<RenderObject>) {
        self.render_object = Some(render_object);
    }
}
