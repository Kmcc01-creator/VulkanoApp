use std::sync::Arc;
use vulkano::device::Device;
use vulkano::pipeline::{graphics::viewport::Viewport, GraphicsPipeline, Pipeline};
use vulkano::render_pass::Subpass;
use vulkano::shader::ShaderModule;

use super::vertex::Vertex;
use crate::core::error::Error;

pub struct PipelineBuilder {
    device: Arc<Device>,
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    viewport: Viewport,
}

impl PipelineBuilder {
    pub fn new(
        device: Arc<Device>,
        vertex_shader: Arc<ShaderModule>,
        fragment_shader: Arc<ShaderModule>,
    ) -> Self {
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        Self {
            device,
            vertex_shader,
            fragment_shader,
            viewport,
        }
    }

    pub fn with_viewport(mut self, viewport: Viewport) -> Self {
        self.viewport = viewport;
        self
    }

    pub fn build(self, subpass: Subpass) -> Result<Arc<GraphicsPipeline>, Error> {
        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(Vertex::per_vertex())
            .vertex_shader(self.vertex_shader.entry_point("main").unwrap(), ())
            .viewport_state(self.viewport.clone())
            .fragment_shader(self.fragment_shader.entry_point("main").unwrap(), ())
            .render_pass(subpass)
            .build(self.device.clone())
            .map_err(|e| Error::GraphicsInitialization(e.to_string()))?;

        Ok(pipeline)
    }
}

pub struct RenderPipeline {
    pipeline: Arc<GraphicsPipeline>,
    viewport: Viewport,
}

impl RenderPipeline {
    pub fn new(pipeline: Arc<GraphicsPipeline>, viewport: Viewport) -> Self {
        Self { pipeline, viewport }
    }

    pub fn raw_pipeline(&self) -> &Arc<GraphicsPipeline> {
        &self.pipeline
    }

    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }
}
