use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::core::Error;
use crate::graphics::shader::{ShaderModule, ShaderType};
use vulkano::device::Device;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub(crate) struct ShaderKey {
    path: PathBuf,
    shader_type: ShaderType,
}

/// A cache system for shader modules to avoid reloading and recompiling shaders
pub struct ShaderCache {
    device: Arc<Device>,
    cache: RwLock<HashMap<ShaderKey, Arc<ShaderModule>>>,
}

impl ShaderCache {
    /// Create a new shader cache using the provided device
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            device,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get a shader from the cache or load it if not present
    pub fn get_or_load<P: AsRef<Path>>(
        &self,
        path: P,
        shader_type: ShaderType,
    ) -> Result<Arc<ShaderModule>, Error> {
        let key = ShaderKey {
            path: path.as_ref().to_path_buf(),
            shader_type,
        };

        // First try to get from cache
        if let Some(shader) = self.cache.read().unwrap().get(&key) {
            return Ok(shader.clone());
        }

        // Load the shader if not in cache
        let shader = Arc::new(crate::load_shader!(
            self.device.clone(),
            path.as_ref().to_str().unwrap(),
            shader_type
        )?);

        self.cache.write().unwrap().insert(key, shader.clone());
        Ok(shader)
    }

    /// Clear all shaders from the cache
    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
    }

    /// Force reload a specific shader
    pub fn reload<P: AsRef<Path>>(
        &self,
        path: P,
        shader_type: ShaderType,
    ) -> Result<Arc<ShaderModule>, Error> {
        let key = ShaderKey {
            path: path.as_ref().to_path_buf(),
            shader_type,
        };

        let shader = Arc::new(crate::load_shader!(
            self.device.clone(),
            path.as_ref().to_str().unwrap(),
            shader_type
        )?);

        self.cache.write().unwrap().insert(key, shader.clone());
        Ok(shader)
    }

    /// Get the underlying device
    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }

    /// Get the number of cached shaders
    pub fn cache_size(&self) -> usize {
        self.cache.read().unwrap().len()
    }

    /// Check if a shader is cached
    pub fn is_cached<P: AsRef<Path>>(&self, path: P, shader_type: ShaderType) -> bool {
        let key = ShaderKey {
            path: path.as_ref().to_path_buf(),
            shader_type,
        };
        self.cache.read().unwrap().contains_key(&key)
    }

    /// Remove a specific shader from the cache
    pub fn remove<P: AsRef<Path>>(&self, path: P, shader_type: ShaderType) {
        let key = ShaderKey {
            path: path.as_ref().to_path_buf(),
            shader_type,
        };
        self.cache.write().unwrap().remove(&key);
    }
}
