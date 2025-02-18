use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    Texture,
    Model,
    Shader,
    Sound,
    Script,
}
pub trait Asset: std::any::Any + 'static + Send + Sync {
    fn asset_type() -> AssetType
    where
        Self: Sized;
    fn path(&self) -> &PathBuf;
    fn clone_box(&self) -> Box<dyn Asset>;
}

// Enable cloning for boxed assets
impl Clone for Box<dyn Asset> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct TextureAsset {
    path: PathBuf,
}

impl TextureAsset {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Asset for TextureAsset {
    fn asset_type() -> AssetType {
        AssetType::Texture
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn clone_box(&self) -> Box<dyn Asset> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct ModelAsset {
    path: PathBuf,
}

impl ModelAsset {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Asset for ModelAsset {
    fn asset_type() -> AssetType {
        AssetType::Model
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn clone_box(&self) -> Box<dyn Asset> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
pub struct ShaderAsset {
    path: PathBuf,
}

impl ShaderAsset {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Asset for ShaderAsset {
    fn asset_type() -> AssetType {
        AssetType::Shader
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn clone_box(&self) -> Box<dyn Asset> {
        Box::new(self.clone())
    }
}
