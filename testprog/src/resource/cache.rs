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

    pub fn get<T: Asset>(&self, path: &Path) -> Option<Arc<T>> {
        self.assets.get(path).and_then(|asset| {
            if asset.as_ref().type_id() == std::any::TypeId::of::<T>() {
                // SAFETY: We just checked that the type matches
                Some(unsafe {
                    let raw = Arc::into_raw(asset.clone());
                    Arc::from_raw(raw as *const T)
                })
            } else {
                None
            }
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
