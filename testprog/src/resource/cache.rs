use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::asset::{Asset, AssetHandle};

pub struct AssetCache {
    assets: HashMap<PathBuf, Box<dyn std::any::Any + Send + Sync>>,
}

impl AssetCache {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn insert<T: Asset>(&mut self, path: PathBuf, asset: T) -> AssetHandle<T> {
        let handle = AssetHandle::new(asset, path.clone());
        self.assets.insert(path, Box::new(handle.clone()));
        handle
    }

    pub fn get<T: Asset>(&self, path: &Path) -> Option<AssetHandle<T>> {
        self.assets
            .get(path)
            .and_then(|boxed| boxed.downcast_ref::<AssetHandle<T>>())
            .cloned()
    }

    pub fn remove(&mut self, path: &Path) -> bool {
        self.assets.remove(path).is_some()
    }

    pub fn clear(&mut self) {
        self.assets.clear();
    }
}

impl Default for AssetCache {
    fn default() -> Self {
        Self::new()
    }
}
