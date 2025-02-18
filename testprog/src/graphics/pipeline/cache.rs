use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use vulkano::device::Device;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout};
use vulkano::render_pass::RenderPass;
use vulkano::shader::ShaderModule;

use crate::core::error::Error;

/// Unique key for identifying pipeline configurations
#[derive(Clone, Eq)]
pub struct PipelineKey {
    shader_stages: Vec<ShaderStageKey>,
    render_pass_hash: u64,
    layout_hash: u64,
    viewport: Option<Viewport>,
    config_hash: u64,
}

#[derive(Clone, Eq, PartialEq)]
struct ShaderStageKey {
    entry_point: String,
    specialization_info: Vec<u8>,
    module_hash: u64,
}

impl Hash for PipelineKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.shader_stages.hash(state);
        self.render_pass_hash.hash(state);
        self.layout_hash.hash(state);
        if let Some(ref viewport) = self.viewport {
            viewport.dimensions[0].to_bits().hash(state);
            viewport.dimensions[1].to_bits().hash(state);
            viewport.origin[0].to_bits().hash(state);
            viewport.origin[1].to_bits().hash(state);
        }
        self.config_hash.hash(state);
    }
}

impl PartialEq for PipelineKey {
    fn eq(&self, other: &Self) -> bool {
        self.shader_stages == other.shader_stages
            && self.render_pass_hash == other.render_pass_hash
            && self.layout_hash == other.layout_hash
            && self.viewport == other.viewport
            && self.config_hash == other.config_hash
    }
}

/// Cache for graphics pipelines
pub struct PipelineCache {
    device: Arc<Device>,
    pipelines: HashMap<PipelineKey, Arc<GraphicsPipeline>>,
    stats: PipelineCacheStats,
}

#[derive(Debug, Default)]
pub struct PipelineCacheStats {
    pub hits: usize,
    pub misses: usize,
    pub total_pipelines: usize,
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
        viewport: Option<Viewport>,
    ) -> Result<Arc<GraphicsPipeline>, Error> {
        // Create pipeline key
        let key = self.create_key(
            &create_info,
            &layout,
            &render_pass,
            shaders,
            viewport.clone(),
        );

        // Check cache
        if let Some(pipeline) = self.pipelines.get(&key) {
            self.stats.hits += 1;
            return Ok(pipeline.clone());
        }

        // Create new pipeline
        self.stats.misses += 1;
        let pipeline =
            GraphicsPipeline::new(self.device.clone(), None, create_info).map_err(|e| {
                Error::GraphicsInitialization(format!("Failed to create pipeline: {}", e))
            })?;

        // Cache pipeline
        self.pipelines.insert(key, pipeline.clone());
        self.stats.total_pipelines = self.pipelines.len();

        Ok(pipeline)
    }

    fn create_key(
        &self,
        create_info: &GraphicsPipelineCreateInfo,
        layout: &Arc<PipelineLayout>,
        render_pass: &Arc<RenderPass>,
        shaders: &[(Arc<ShaderModule>, &str)],
        viewport: Option<Viewport>,
    ) -> PipelineKey {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        create_info.hash(&mut hasher);
        let config_hash = hasher.finish();

        let shader_stages = shaders
            .iter()
            .map(|(module, entry_point)| {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                module.as_ref().hash(&mut hasher);
                ShaderStageKey {
                    entry_point: entry_point.to_string(),
                    specialization_info: Vec::new(), // Add specialization if needed
                    module_hash: hasher.finish(),
                }
            })
            .collect();

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        render_pass.as_ref().hash(&mut hasher);
        let render_pass_hash = hasher.finish();

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        layout.as_ref().hash(&mut hasher);
        let layout_hash = hasher.finish();

        PipelineKey {
            shader_stages,
            render_pass_hash,
            layout_hash,
            viewport,
            config_hash,
        }
    }

    pub fn clear(&mut self) {
        self.pipelines.clear();
        self.stats = PipelineCacheStats::default();
    }

    pub fn get_stats(&self) -> &PipelineCacheStats {
        &self.stats
    }

    pub fn invalidate_shaders(&mut self, modules: &[Arc<ShaderModule>]) {
        let module_hashes: Vec<u64> = modules
            .iter()
            .map(|module| {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                module.as_ref().hash(&mut hasher);
                hasher.finish()
            })
            .collect();

        self.pipelines.retain(|key, _| {
            !key.shader_stages
                .iter()
                .any(|stage| module_hashes.contains(&stage.module_hash))
        });

        self.stats.total_pipelines = self.pipelines.len();
    }
}

// Helper function to compute hash for arbitrary pipeline configuration data
pub fn compute_config_hash<T: Hash>(config: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    config.hash(&mut hasher);
    hasher.finish()
}
