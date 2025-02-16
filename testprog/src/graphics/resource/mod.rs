mod shader_cache;
mod material;

pub use shader_cache::ShaderCache;
pub use material::{Material, MaterialBuilder, MaterialProperties};

// Re-export key types for internal use
pub(crate) use shader_cache::ShaderKey;

/// Example usage of the material system:
/// ```ignore
/// let shader_cache = ShaderCache::new(device.clone());
/// 
/// // Create a basic material
/// let material = MaterialBuilder::new(device.clone(), shader_cache.clone())
///     .with_vertex_shader("shaders/basic.vert.spv")
///     .with_fragment_shader("shaders/basic.frag.spv")
///     .with_properties(MaterialProperties {
///         albedo: [1.0, 0.0, 0.0, 1.0],  // Red color
///         metallic: 0.0,
///         roughness: 0.5,
///         ambient_occlusion: 1.0,
///     })
///     .build_3d(render_pass.clone())?;
