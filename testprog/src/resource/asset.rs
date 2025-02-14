use std::any::TypeId;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    Texture,
    Model,
    Shader,
    Sound,
    Script,
}

pub trait Asset: 'static + Send + Sync {
    fn asset_type(&self) -> AssetType;
    fn path(&self) -> &PathBuf;
}

#[derive(Clone)]
pub struct AssetHandle<T: Asset> {
    asset: Arc<T>,
    path: PathBuf,
}

impl<T: Asset> AssetHandle<T> {
    pub fn new(asset: T, path: PathBuf) -> Self {
        Self {
            asset: Arc::new(asset),
            path,
        }
    }

    pub fn get(&self) -> &T {
        &self.asset
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn clone_inner(&self) -> Arc<T> {
        self.asset.clone()
    }
}

impl<T: Asset> std::fmt::Debug for AssetHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetHandle")
            .field("type", &std::any::type_name::<T>())
            .field("path", &self.path)
            .finish()
    }
}

// Common asset types
pub struct TextureAsset {
    path: PathBuf,
    // Add texture-specific fields
}

impl Asset for TextureAsset {
    fn asset_type(&self) -> AssetType {
        AssetType::Texture
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }
}

pub struct ModelAsset {
    path: PathBuf,
    // Add model-specific fields
}

impl Asset for ModelAsset {
    fn asset_type(&self) -> AssetType {
        AssetType::Model
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }
}

pub struct ShaderAsset {
    path: PathBuf,
    // Add shader-specific fields
}

impl Asset for ShaderAsset {
    fn asset_type(&self) -> AssetType {
        AssetType::Shader
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }
}
