use glam::Vec4;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    pub max_frames_in_flight: u32,
    pub msaa_samples: u32,
    pub texture_memory_budget: u32, // In megabytes
    pub max_cached_textures: u32,
    pub anisotropic_filtering: u32,
    pub camera: CameraConfig,
    pub render_settings: RenderSettings,
    pub debug: DebugSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    pub fov: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub clear_color: Vec4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderSettings {
    pub enable_shadows: bool,
    pub shadow_map_size: u32,
    pub enable_bloom: bool,
    pub bloom_threshold: f32,
    pub enable_ssao: bool,
    pub ssao_radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSettings {
    pub enable_validation: bool,
    pub enable_profiling: bool,
    pub debug_markers: bool,
    pub pipeline_statistics: bool,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            max_frames_in_flight: 2,
            msaa_samples: 1,
            texture_memory_budget: 512, // 512MB
            max_cached_textures: 1000,
            anisotropic_filtering: 8,
            camera: CameraConfig::default(),
            render_settings: RenderSettings::default(),
            debug: DebugSettings::default(),
        }
    }
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            fov: std::f32::consts::PI / 4.0,
            near_plane: 0.1,
            far_plane: 1000.0,
            clear_color: Vec4::new(0.1, 0.1, 0.1, 1.0),
        }
    }
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            enable_shadows: true,
            shadow_map_size: 2048,
            enable_bloom: true,
            bloom_threshold: 1.0,
            enable_ssao: true,
            ssao_radius: 0.5,
        }
    }
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            enable_validation: cfg!(debug_assertions),
            enable_profiling: cfg!(debug_assertions),
            debug_markers: cfg!(debug_assertions),
            pipeline_statistics: cfg!(debug_assertions),
        }
    }
}

pub struct GraphicsConfigBuilder {
    config: GraphicsConfig,
}

impl GraphicsConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: GraphicsConfig::default(),
        }
    }

    pub fn max_frames_in_flight(mut self, frames: u32) -> Self {
        self.config.max_frames_in_flight = frames;
        self
    }

    pub fn msaa_samples(mut self, samples: u32) -> Self {
        self.config.msaa_samples = samples;
        self
    }

    pub fn texture_memory_budget(mut self, budget_mb: u32) -> Self {
        self.config.texture_memory_budget = budget_mb;
        self
    }

    pub fn max_cached_textures(mut self, max: u32) -> Self {
        self.config.max_cached_textures = max;
        self
    }

    pub fn anisotropic_filtering(mut self, level: u32) -> Self {
        self.config.anisotropic_filtering = level;
        self
    }

    pub fn camera_settings(mut self, fov: f32, near: f32, far: f32, clear_color: Vec4) -> Self {
        self.config.camera = CameraConfig {
            fov,
            near_plane: near,
            far_plane: far,
            clear_color,
        };
        self
    }

    pub fn render_settings(mut self, settings: RenderSettings) -> Self {
        self.config.render_settings = settings;
        self
    }

    pub fn debug_settings(mut self, settings: DebugSettings) -> Self {
        self.config.debug = settings;
        self
    }

    pub fn build(self) -> GraphicsConfig {
        self.config
    }
}

impl Default for GraphicsConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
