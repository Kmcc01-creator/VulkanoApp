use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use vulkano::device::Device;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::graphics::{
    input_assembly::InputAssemblyState,
    input_assembly::PrimitiveTopology,
    vertex_input::VertexInputState,
    viewport::{Scissor, ViewportState},
    GraphicsPipelineCreateInfo,
};
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout};
use vulkano::render_pass::RenderPass;
use vulkano::shader::{ShaderModule, ShaderStages};

use crate::core::error::Error;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct ShaderStageKey {
    entry_point: String,
}

#[derive(Debug, Clone)]
pub struct PipelineKey {
    shader_stages: Vec<ShaderStageKey>,
    viewport: Option<Viewport>,
}

impl Hash for PipelineKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash shader stages
        for stage in &self.shader_stages {
            stage.hash(state);
        }

        // Hash viewport if present
        if let Some(viewport) = &self.viewport {
            // Get viewport dimensions and ranges
            for val in viewport.offset.iter() {
                val.to_bits().hash(state);
            }
            for val in viewport.extent.iter() {
                val.to_bits().hash(state);
            }
            let range = &viewport.depth_range;
            range.clone().start().to_bits().hash(state);
            range.clone().end().to_bits().hash(state);
        }
    }
}

impl PartialEq for PipelineKey {
    fn eq(&self, other: &Self) -> bool {
        self.shader_stages == other.shader_stages && self.viewport == other.viewport
    }
}

impl Eq for PipelineKey {}

#[derive(Debug, Default)]
pub struct PipelineCacheStats {
    pub hits: usize,
    pub misses: usize,
    pub cached_pipelines: usize,
}

pub struct PipelineCache {
    device: Arc<Device>,
    pipelines: HashMap<PipelineKey, Arc<GraphicsPipeline>>,
    stats: PipelineCacheStats,
}

impl PipelineCache {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            device,
            pipelines: HashMap::new(),
            stats: PipelineCacheStats::default(),
        }
    }

    pub fn get_or_create(
        &mut self,
        create_info: GraphicsPipelineCreateInfo,
        layout: Arc<PipelineLayout>,
        render_pass: Arc<RenderPass>,
        shaders: &[(Arc<ShaderModule>, &str)],
        _viewport: Option<Viewport>,
    ) -> Result<Arc<GraphicsPipeline>, Error> {
        // Create key from shader info
        let key = PipelineKey {
            shader_stages: shaders
                .iter()
                .map(|(_, entry)| ShaderStageKey {
                    entry_point: entry.to_string(),
                })
                .collect(),
            viewport: _viewport,
        };

        // Check cache
        if let Some(pipeline) = self.pipelines.get(&key) {
            self.stats.hits += 1;
            return Ok(pipeline.clone());
        }

        // Create new pipeline
        self.stats.misses += 1;

        // Note: GraphicsPipeline::new returns Arc<GraphicsPipeline>
        let result = unsafe { GraphicsPipeline::new(self.device.clone(), None, create_info) };

        let pipeline = result.map_err(|e| {
            Error::GraphicsInitialization(format!("Failed to create pipeline: {}", e))
        })?;
        self.pipelines.insert(key, pipeline.clone());
        self.stats.cached_pipelines = self.pipelines.len();

        Ok(pipeline)
    }

    pub fn invalidate_shaders(&mut self, shaders: &[(Arc<ShaderModule>, &str)]) {
        let entry_points: Vec<_> = shaders.iter().map(|(_, entry)| entry.to_string()).collect();
        self.pipelines.retain(|key, _| {
            !key.shader_stages
                .iter()
                .any(|stage| entry_points.contains(&stage.entry_point))
        });
        self.stats.cached_pipelines = self.pipelines.len();
    }

    pub fn clear(&mut self) {
        self.pipelines.clear();
        self.stats.cached_pipelines = 0;
    }

    pub fn get_stats(&self) -> &PipelineCacheStats {
        &self.stats
    }
}
