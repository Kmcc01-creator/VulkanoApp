use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::asset::Asset;

pub struct AssetCache {
    assets: HashMap<PathBuf, Box<dyn Asset>>,
}

impl AssetCache {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn insert<T>(&mut self, path: PathBuf, asset: T) -> Box<dyn Asset>
    where
        T: Asset + 'static,
    {
        let boxed = Box::new(asset) as Box<dyn Asset>;
        self.assets.insert(path, boxed.clone());
        boxed
    }

    pub fn get<T: Asset + 'static>(&self, path: &Path) -> Option<Box<T>> {
        self.assets.get(path).and_then(|asset| {
            asset
                .as_any()
                .downcast_ref::<T>()
                .map(|a| Box::new(a.clone()))
        })
    }

    pub fn get_raw(&self, path: &Path) -> Option<Box<dyn Asset>> {
        self.assets.get(path).map(|asset| asset.clone())
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
