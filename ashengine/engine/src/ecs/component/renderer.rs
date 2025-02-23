//! Renderer component for ECS integration
//!
//! Provides a unified renderer component that works with the graphics system.

use crate::graphics::{render::PassType, resource::ResourceHandle};

/// Base renderer component shared by all renderer types
#[derive(Debug, Clone)]
pub struct RenderComponent {
    /// Whether the entity should be rendered
    pub visible: bool,
    /// Render layer for sorting
    pub layer: i32,
    /// Pass type this renderer should be processed in
    pub pass_type: PassType,
    /// Mesh resource handle
    mesh: ResourceHandle,
    /// Material handle
    material: Option<ResourceHandle>,
    /// Transform buffer handle
    transform_buffer: ResourceHandle,
}

impl RenderComponent {
    /// Create a new renderer component
    pub fn new(mesh: ResourceHandle, transform_buffer: ResourceHandle) -> Self {
        Self {
            visible: true,
            layer: 0,
            pass_type: PassType::Geometry,
            mesh,
            material: None,
            transform_buffer,
        }
    }

    /// Set the material for this renderer
    pub fn with_material(mut self, material: ResourceHandle) -> Self {
        self.material = Some(material);
        self
    }

    /// Set the render layer
    pub fn with_layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    /// Set the pass type
    pub fn with_pass_type(mut self, pass_type: PassType) -> Self {
        self.pass_type = pass_type;
        self
    }

    /// Check if this renderer should be processed
    pub fn should_render(&self) -> bool {
        self.visible
    }

    /// Get the mesh resource
    pub fn mesh(&self) -> ResourceHandle {
        self.mesh
    }

    /// Get the material resource
    pub fn material(&self) -> Option<ResourceHandle> {
        self.material
    }

    /// Get the transform buffer
    pub fn transform_buffer(&self) -> ResourceHandle {
        self.transform_buffer
    }

    /// Get the pass type
    pub fn pass_type(&self) -> PassType {
        self.pass_type
    }

    /// Get the sort key for ordering within a pass
    pub fn sort_key(&self) -> i32 {
        self.layer
    }
}

/// Component for static mesh rendering
#[derive(Debug, Clone)]
pub struct StaticMeshRenderer {
    base: RenderComponent,
    enable_culling: bool,
}

impl StaticMeshRenderer {
    /// Create a new static mesh renderer
    pub fn new(mesh: ResourceHandle, transform_buffer: ResourceHandle) -> Self {
        Self {
            base: RenderComponent::new(mesh, transform_buffer),
            enable_culling: true,
        }
    }

    /// Set the material
    pub fn with_material(mut self, material: ResourceHandle) -> Self {
        self.base = self.base.with_material(material);
        self
    }

    /// Set the render layer
    pub fn with_layer(mut self, layer: i32) -> Self {
        self.base = self.base.with_layer(layer);
        self
    }

    /// Set whether frustum culling is enabled
    pub fn with_culling(mut self, enable: bool) -> Self {
        self.enable_culling = enable;
        self
    }

    /// Get the base render component
    pub fn base(&self) -> &RenderComponent {
        &self.base
    }

    /// Check if frustum culling is enabled
    pub fn culling_enabled(&self) -> bool {
        self.enable_culling
    }
}

/// Component for UI element rendering
#[derive(Debug, Clone)]
pub struct UIRenderer {
    base: RenderComponent,
    color: [f32; 4],
}

impl UIRenderer {
    /// Create a new UI renderer
    pub fn new(mesh: ResourceHandle, transform_buffer: ResourceHandle) -> Self {
        let mut base = RenderComponent::new(mesh, transform_buffer);
        base.pass_type = PassType::UI;

        Self {
            base,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    /// Set the material
    pub fn with_material(mut self, material: ResourceHandle) -> Self {
        self.base = self.base.with_material(material);
        self
    }

    /// Set the color
    pub fn with_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = [r, g, b, a];
        self
    }

    /// Get the base render component
    pub fn base(&self) -> &RenderComponent {
        &self.base
    }

    /// Get the color
    pub fn color(&self) -> [f32; 4] {
        self.color
    }
}
