mod asset;
mod cache;
mod loader;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub use asset::{Asset, AssetHandle, AssetType};
pub use cache::AssetCache;
pub use loader::AssetLoader;

use crate::core::error::Error;

pub struct ResourceManager {
    asset_path: PathBuf,
    loaders: HashMap<AssetType, Box<dyn AssetLoader>>,
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

    pub fn load_asset<T: Asset>(&mut self, path: &str) -> Result<AssetHandle<T>, Error> {
        let asset_type = T::asset_type();
        let full_path = self.asset_path.join(path);

        if let Some(cached) = self.cache.get(&full_path) {
            return Ok(cached.clone());
        }

        let loader = self.loaders.get(&asset_type).ok_or_else(|| {
            Error::ResourceError(format!("No loader for asset type {:?}", asset_type))
        })?;

        let asset = loader.load(&full_path)?;
        let handle = self.cache.insert(full_path, asset);

        Ok(handle)
    }

    pub fn register_loader<L: AssetLoader + 'static>(&mut self, loader: L) {
        self.loaders.insert(loader.asset_type(), Box::new(loader));
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}
