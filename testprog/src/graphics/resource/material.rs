use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuBufferPool, TypedBufferAccess};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::Device;
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::depth_stencil::DepthStencilState;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::{CullMode, FrontFace, RasterizationState};
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::PipelineLayout;

use super::shader_cache::ShaderCache;
use crate::core::Error;
use crate::graphics::shader::ShaderModule;
use crate::graphics::vertex::{Vertex2D, Vertex3D};

#[derive(Debug, Clone, Copy)]
pub struct MaterialProperties {
    pub albedo: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub ambient_occlusion: f32,
}

impl Default for MaterialProperties {
    fn default() -> Self {
        Self {
            albedo: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            ambient_occlusion: 1.0,
        }
    }
}

#[derive(Clone)]
pub struct Material {
    pipeline: Arc<GraphicsPipeline>,
    descriptor_set: Arc<PersistentDescriptorSet>,
    properties: MaterialProperties,
}

impl Material {
    pub fn pipeline(&self) -> &Arc<GraphicsPipeline> {
        &self.pipeline
    }

    pub fn descriptor_set(&self) -> &Arc<PersistentDescriptorSet> {
        &self.descriptor_set
    }

    pub fn properties(&self) -> &MaterialProperties {
        &self.properties
    }
}

pub struct MaterialBuilder {
    device: Arc<Device>,
    shader_cache: Arc<ShaderCache>,
    vertex_shader: Option<String>,
    fragment_shader: Option<String>,
    properties: MaterialProperties,
}

impl MaterialBuilder {
    pub fn new(device: Arc<Device>, shader_cache: Arc<ShaderCache>) -> Self {
        Self {
            device,
            shader_cache,
            vertex_shader: None,
            fragment_shader: None,
            properties: MaterialProperties::default(),
        }
    }

    pub fn with_vertex_shader<P: Into<String>>(mut self, path: P) -> Self {
        self.vertex_shader = Some(path.into());
        self
    }

    pub fn with_fragment_shader<P: Into<String>>(mut self, path: P) -> Self {
        self.fragment_shader = Some(path.into());
        self
    }

    pub fn with_properties(mut self, properties: MaterialProperties) -> Self {
        self.properties = properties;
        self
    }

    pub fn build_2d(
        &self,
        render_pass: Arc<vulkano::render_pass::RenderPass>,
    ) -> Result<Material, Error> {
        self.build_internal::<Vertex2D>(render_pass)
    }

    pub fn build_3d(
        &self,
        render_pass: Arc<vulkano::render_pass::RenderPass>,
    ) -> Result<Material, Error> {
        self.build_internal::<Vertex3D>(render_pass)
    }

    fn build_internal<V: vulkano::pipeline::graphics::vertex_input::Vertex>(
        &self,
        render_pass: Arc<vulkano::render_pass::RenderPass>,
    ) -> Result<Material, Error> {
        let vertex_shader = self
            .vertex_shader
            .as_ref()
            .ok_or_else(|| Error::GraphicsInitialization("Vertex shader not specified".into()))?;
        let fragment_shader = self
            .fragment_shader
            .as_ref()
            .ok_or_else(|| Error::GraphicsInitialization("Fragment shader not specified".into()))?;

        // Load shaders
        let vert_shader = self
            .shader_cache
            .get_or_load(vertex_shader, crate::graphics::shader::ShaderType::Vertex)?;
        let frag_shader = self.shader_cache.get_or_load(
            fragment_shader,
            crate::graphics::shader::ShaderType::Fragment,
        )?;

        // Create uniform buffer for material properties
        let uniform_pool = CpuBufferPool::<MaterialProperties>::uniform_buffer(self.device.clone());
        let uniform_subbuffer = uniform_pool.next(self.properties).map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to create uniform buffer: {}", e))
        })?;

        // Pipeline layout creation is now handled by the pipeline builder

        // Create pipeline
        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(V::per_vertex())
            .vertex_shader(vert_shader.as_ref().clone(), ())
            .input_assembly_state(InputAssemblyState::new())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .rasterization_state(RasterizationState::new())
            .multisample_state(MultisampleState::new())
            .depth_stencil_state(DepthStencilState::simple_depth_test())
            .fragment_shader(frag_shader.as_ref().clone(), ())
            .color_blend_state(ColorBlendState::new(render_pass.num_subpasses()))
            .render_pass(render_pass.clone(), 0)
            .build(self.device.clone())
            .map_err(|e| {
                Error::GraphicsInitialization(format!("Failed to create graphics pipeline: {}", e))
            })?;

        // Create descriptor set
        let layout = pipeline
            .layout()
            .set_layouts()
            .get(0)
            .ok_or_else(|| {
                Error::GraphicsInitialization("Pipeline missing descriptor set layout".into())
            })?
            .clone();

        let descriptor_set = PersistentDescriptorSet::start(layout)
            .add_buffer(uniform_subbuffer)
            .map_err(|e| {
                Error::GraphicsInitialization(format!(
                    "Failed to add buffer to descriptor set: {}",
                    e
                ))
            })?
            .build()
            .map_err(|e| {
                Error::GraphicsInitialization(format!("Failed to build descriptor set: {}", e))
            })?;

        Ok(Material {
            pipeline,
            descriptor_set: Arc::new(descriptor_set),
            properties: self.properties,
        })
    }
}
