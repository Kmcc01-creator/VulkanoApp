mod cache;

use std::sync::Arc;
use vulkano::{
    device::Device,
    pipeline::{
        graphics::GraphicsPipelineCreateInfo, layout::PipelineLayoutCreateInfo, GraphicsPipeline,
        PipelineLayout,
    },
    render_pass::RenderPass,
    shader::ShaderModule,
    VulkanObject,
};

use crate::core::error::Error;
pub use cache::{PipelineCache, PipelineCacheStats};

pub struct PipelineManager {
    device: Arc<Device>,
    cache: PipelineCache,
}

impl PipelineManager {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            device: device.clone(),
            cache: PipelineCache::new(device),
        }
    }

    pub fn create_pipeline(
        &mut self,
        create_info: GraphicsPipelineCreateInfo,
        layout: Arc<PipelineLayout>,
        render_pass: Arc<RenderPass>,
        shaders: &[(Arc<ShaderModule>, &str)],
        viewport: Option<Viewport>,
    ) -> Result<Arc<GraphicsPipeline>, Error> {
        self.cache
            .get_or_create(create_info, layout, render_pass, shaders, viewport)
    }

    pub fn invalidate_shaders(&mut self, modules: &[Arc<ShaderModule>]) {
        let modules_with_entry = modules
            .iter()
            .map(|m| (m.clone(), "main"))
            .collect::<Vec<_>>();
        self.cache.invalidate_shaders(&modules_with_entry);
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn get_stats(&self) -> &PipelineCacheStats {
        self.cache.get_stats()
    }
}

/// Configuration for pipeline creation
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub vertex_shader: Arc<ShaderModule>,
    pub fragment_shader: Arc<ShaderModule>,
    pub vertex_entry: String,
    pub fragment_entry: String,
    pub viewport: Option<Viewport>,
}

/// Helper for creating common pipeline configurations
pub struct PipelineBuilder {
    device: Arc<Device>,
    config: PipelineConfig,
    create_info: GraphicsPipelineCreateInfo,
}

impl PipelineBuilder {
    pub fn new(
        device: Arc<Device>,
        vertex_shader: Arc<ShaderModule>,
        fragment_shader: Arc<ShaderModule>,
    ) -> Self {
        Self {
            device: device.clone(),
            config: PipelineConfig {
                vertex_shader,
                fragment_shader,
                vertex_entry: "main".to_string(),
                fragment_entry: "main".to_string(),
                viewport: None,
            },
            create_info: GraphicsPipelineCreateInfo::layout(
                PipelineLayout::new(device, PipelineLayoutCreateInfo::default()).unwrap(),
            ),
        }
    }

    pub fn vertex_entry(mut self, entry: &str) -> Self {
        self.config.vertex_entry = entry.to_string();
        self
    }

    pub fn fragment_entry(mut self, entry: &str) -> Self {
        self.config.fragment_entry = entry.to_string();
        self
    }

    pub fn viewport(mut self, viewport: Viewport) -> Self {
        self.config.viewport = Some(viewport);
        self
    }

    pub fn build(
        self,
        manager: &mut PipelineManager,
        layout: Arc<PipelineLayout>,
        render_pass: Arc<RenderPass>,
    ) -> Result<Arc<GraphicsPipeline>, Error> {
        let shaders: Vec<(Arc<ShaderModule>, &str)> = vec![
            (
                self.config.vertex_shader.clone(),
                self.config.vertex_entry.as_str(),
            ),
            (
                self.config.fragment_shader.clone(),
                self.config.fragment_entry.as_str(),
            ),
        ];

        manager.create_pipeline(
            self.create_info,
            layout,
            render_pass,
            &shaders,
            self.config.viewport,
        )
    }
}

// Re-export common pipeline types
pub use vulkano::pipeline::graphics::{
    color_blend::{ColorBlendAttachmentState, ColorBlendState},
    depth_stencil::DepthStencilState,
    input_assembly::InputAssemblyState,
    multisample::MultisampleState,
    rasterization::RasterizationState,
    vertex_input::VertexInputState,
    viewport::{Scissor, Viewport, ViewportState},
};
