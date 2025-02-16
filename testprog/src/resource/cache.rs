use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::asset::Asset;

pub struct AssetCache {
    assets: HashMap<PathBuf, Arc<dyn Asset>>,
}

impl AssetCache {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn insert<T>(&mut self, path: PathBuf, asset: T) -> Arc<T>
    where
        T: Asset + 'static,
    {
        let arc = Arc::new(asset);
        self.assets.insert(path, arc.clone() as Arc<dyn Asset>);
        arc
    }

    pub fn get<T: Asset + 'static>(&self, path: &Path) -> Option<Arc<T>> {
        self.assets.get(path).and_then(|asset| {
            asset.as_any().downcast_ref::<T>().map(|_| {
                // Since we know the type matches, we can safely clone the Arc
                // and downcast it
                Arc::downcast::<T>(asset.clone()).expect("Downcast failed after type check")
            })
        })
    }

    pub fn get_raw(&self, path: &Path) -> Option<Arc<dyn Asset>> {
        self.assets.get(path).map(Arc::clone)
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
