mod asset;
mod cache;
mod loader;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub use asset::{Asset, AssetType};
pub use cache::AssetCache;
use loader::AssetLoader;

use crate::core::error::Error;

pub struct ResourceManager {
    asset_path: PathBuf,
    loaders: HashMap<AssetType, AssetLoader>,
    cache: AssetCache,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            asset_path: PathBuf::from("assets"),
            loaders: HashMap::new(),
            cache: AssetCache::new(),
        }
    }

    pub fn load_asset<T: Asset + 'static>(&mut self, path: &str) -> Result<Arc<T>, Error> {
        let full_path = self.asset_path.join(path);

        // Try to get from cache first
        if let Some(asset) = self.cache.get::<T>(&full_path) {
            return Ok(asset);
        }

        // Load using appropriate loader
        let loader = self.loaders.get(&T::asset_type()).ok_or_else(|| {
            Error::ResourceError(format!("No loader for asset type {:?}", T::asset_type()))
        })?;

        let typed_asset = loader.load_typed::<T>(&full_path)?;

        Ok(self.cache.insert(full_path, typed_asset))
    }

    pub fn register_loader(&mut self, loader: AssetLoader) {
        self.loaders.insert(loader.asset_type(), loader);
    }

    pub fn set_asset_path<P: Into<PathBuf>>(&mut self, path: P) {
        self.asset_path = path.into();
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}
