use super::asset::{Asset, AssetType};
use crate::core::error::Error;
use std::path::Path;

pub trait AssetLoader: Send + Sync {
    fn asset_type(&self) -> AssetType;
    fn load(&self, path: &Path) -> Result<Box<dyn Asset>, Error>;
    fn extensions(&self) -> &[&str];
}

pub struct TextureLoader;
impl AssetLoader for TextureLoader {
    fn asset_type(&self) -> AssetType {
        AssetType::Texture
    }

    fn load(&self, _path: &Path) -> Result<Box<dyn Asset>, Error> {
        // TODO: Implement texture loading using vulkan
        unimplemented!("Texture loading not yet implemented")
    }

    fn extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg"]
    }
}

pub struct ShaderLoader;
impl AssetLoader for ShaderLoader {
    fn asset_type(&self) -> AssetType {
        AssetType::Shader
    }

    fn load(&self, _path: &Path) -> Result<Box<dyn Asset>, Error> {
        // TODO: Implement shader loading using vulkan
        unimplemented!("Shader loading not yet implemented")
    }

    fn extensions(&self) -> &[&str] {
        &["vert", "frag", "comp"]
    }
}

pub struct ModelLoader;
impl AssetLoader for ModelLoader {
    fn asset_type(&self) -> AssetType {
        AssetType::Model
    }

    fn load(&self, _path: &Path) -> Result<Box<dyn Asset>, Error> {
        // TODO: Implement model loading
        unimplemented!("Model loading not yet implemented")
    }

    fn extensions(&self) -> &[&str] {
        &["obj", "fbx", "gltf"]
    }
}
