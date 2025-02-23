//! Renderer archetypes for common rendering scenarios
//!
//! Provides pre-configured renderer setups for common use cases like
//! static meshes, skinned characters, particles, and UI elements.

use crate::{
    ecs::component::renderer::{
        ParticleRenderer, SkinnedMeshRenderer, StaticMeshRenderer, UIRenderer,
    },
    error::Result,
    graphics::{
        render::{DepthConfig, PassType, PipelineBuilder, RasterizationConfig},
        resource::{MaterialParam, ResourceHandle, ResourceManager, ShaderStage},
    },
};
use std::sync::Arc;

/// Configuration for creating a static mesh renderer
pub struct StaticMeshConfig {
    pub mesh: ResourceHandle,
    pub albedo_texture: Option<ResourceHandle>,
    pub normal_texture: Option<ResourceHandle>,
    pub metallic_roughness_texture: Option<ResourceHandle>,
    pub enable_culling: bool,
    pub cast_shadows: bool,
}

/// Configuration for creating a skinned mesh renderer
pub struct SkinnedMeshConfig {
    pub mesh: ResourceHandle,
    pub skeleton: ResourceHandle,
    pub albedo_texture: Option<ResourceHandle>,
    pub normal_texture: Option<ResourceHandle>,
    pub cast_shadows: bool,
}

/// Configuration for creating a particle renderer
pub struct ParticleConfig {
    pub max_particles: u32,
    pub texture: Option<ResourceHandle>,
    pub additive_blending: bool,
}

/// Configuration for creating a UI renderer
pub struct UIConfig {
    pub texture: Option<ResourceHandle>,
    pub color: [f32; 4],
    pub layer: i32,
}

/// Factory for creating renderer components with standard configurations
pub struct RendererFactory {
    resource_manager: Arc<ResourceManager>,
    default_quad_mesh: ResourceHandle,
    default_materials: DefaultMaterials,
}

/// Collection of default materials for different renderer types
struct DefaultMaterials {
    pbr: ResourceHandle,
    skinned: ResourceHandle,
    particle: ResourceHandle,
    ui: ResourceHandle,
}

impl RendererFactory {
    /// Create a new renderer factory
    pub fn new(resource_manager: Arc<ResourceManager>) -> Result<Self> {
        // Create default quad mesh for particles and UI
        let quad_mesh = Self::create_quad_mesh(&resource_manager)?;

        // Create default materials
        let default_materials = Self::create_default_materials(&resource_manager)?;

        Ok(Self {
            resource_manager,
            default_quad_mesh: quad_mesh,
            default_materials,
        })
    }

    /// Create a static mesh renderer
    pub fn create_static_mesh(&self, config: StaticMeshConfig) -> Result<StaticMeshRenderer> {
        // Create PBR material if textures are provided
        let material = if config.albedo_texture.is_some() {
            let material = self.create_pbr_material(
                config.albedo_texture,
                config.normal_texture,
                config.metallic_roughness_texture,
            )?;
            Some(material)
        } else {
            Some(self.default_materials.pbr)
        };

        let mut renderer = StaticMeshRenderer::new(config.mesh);
        renderer.enable_culling = config.enable_culling;

        if let Some(mat) = material {
            renderer = renderer.with_material(mat);
        }

        Ok(renderer)
    }

    /// Create a skinned mesh renderer
    pub fn create_skinned_mesh(&self, config: SkinnedMeshConfig) -> Result<SkinnedMeshRenderer> {
        // Create skinned material if textures are provided
        let material = if config.albedo_texture.is_some() {
            let material =
                self.create_skinned_material(config.albedo_texture, config.normal_texture)?;
            Some(material)
        } else {
            Some(self.default_materials.skinned)
        };

        let mut renderer = SkinnedMeshRenderer::new(config.mesh, config.skeleton);

        if let Some(mat) = material {
            renderer = renderer.with_material(mat);
        }

        Ok(renderer)
    }

    /// Create a particle renderer
    pub fn create_particle_system(&self, config: ParticleConfig) -> Result<ParticleRenderer> {
        // Create particle buffer
        let particle_buffer = self.resource_manager.create_buffer(
            (std::mem::size_of::<f32>() * 8 * config.max_particles) as u64,
            ash::vk::BufferUsageFlags::STORAGE_BUFFER | ash::vk::BufferUsageFlags::TRANSFER_DST,
            crate::graphics::resource::BufferType::Storage,
        )?;

        let material = if config.texture.is_some() {
            let material =
                self.create_particle_material(config.texture, config.additive_blending)?;
            Some(material)
        } else {
            Some(self.default_materials.particle)
        };

        let mut renderer = ParticleRenderer::new(particle_buffer, config.max_particles);

        if let Some(mat) = material {
            renderer = renderer.with_material(mat);
        }

        Ok(renderer)
    }

    /// Create a UI renderer
    pub fn create_ui_element(&self, config: UIConfig) -> Result<UIRenderer> {
        let material = if config.texture.is_some() {
            let material = self.create_ui_material(config.texture)?;
            Some(material)
        } else {
            Some(self.default_materials.ui)
        };

        let mut renderer = UIRenderer::new(self.default_quad_mesh).with_color(
            config.color[0],
            config.color[1],
            config.color[2],
            config.color[3],
        );

        renderer.base.layer = config.layer;

        if let Some(mat) = material {
            renderer = renderer.with_material(mat);
        }

        Ok(renderer)
    }

    // Helper functions for creating materials
    fn create_pbr_material(
        &self,
        albedo: Option<ResourceHandle>,
        normal: Option<ResourceHandle>,
        metallic_roughness: Option<ResourceHandle>,
    ) -> Result<ResourceHandle> {
        // TODO: Implement material creation with the resource manager
        Ok(self.default_materials.pbr)
    }

    fn create_skinned_material(
        &self,
        albedo: Option<ResourceHandle>,
        normal: Option<ResourceHandle>,
    ) -> Result<ResourceHandle> {
        // TODO: Implement material creation with the resource manager
        Ok(self.default_materials.skinned)
    }

    fn create_particle_material(
        &self,
        texture: Option<ResourceHandle>,
        additive: bool,
    ) -> Result<ResourceHandle> {
        // TODO: Implement material creation with the resource manager
        Ok(self.default_materials.particle)
    }

    fn create_ui_material(&self, texture: Option<ResourceHandle>) -> Result<ResourceHandle> {
        // TODO: Implement material creation with the resource manager
        Ok(self.default_materials.ui)
    }

    // Helper function to create default materials
    fn create_default_materials(resource_manager: &ResourceManager) -> Result<DefaultMaterials> {
        // TODO: Implement default material creation
        Ok(DefaultMaterials {
            pbr: ResourceHandle::default(),
            skinned: ResourceHandle::default(),
            particle: ResourceHandle::default(),
            ui: ResourceHandle::default(),
        })
    }

    // Helper function to create a quad mesh
    fn create_quad_mesh(resource_manager: &ResourceManager) -> Result<ResourceHandle> {
        // TODO: Implement quad mesh creation
        Ok(ResourceHandle::default())
    }
}
