use super::asset::{Asset, AssetType, ModelAsset, ShaderAsset, TextureAsset};
use crate::core::error::Error;
use std::any::Any;
use std::path::Path;

pub trait AssetLoader: Send + Sync {
    fn asset_type(&self) -> AssetType;
    fn load(&self, path: &Path) -> Result<Box<dyn Any + Send + Sync>, Error>;
    fn extensions(&self) -> &[&str];
}

pub struct TextureLoader;
impl AssetLoader for TextureLoader {
    fn asset_type(&self) -> AssetType {
        TextureAsset::asset_type()
    }

    fn load(&self, path: &Path) -> Result<Box<dyn Any + Send + Sync>, Error> {
        let asset = TextureAsset::new(path.to_owned());
        Ok(Box::new(asset))
    }

    fn extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg"]
    }
}

pub struct ShaderLoader;
impl AssetLoader for ShaderLoader {
    fn asset_type(&self) -> AssetType {
        ShaderAsset::asset_type()
    }

    fn load(&self, path: &Path) -> Result<Box<dyn Any + Send + Sync>, Error> {
        let asset = ShaderAsset::new(path.to_owned());
        Ok(Box::new(asset))
    }

    fn extensions(&self) -> &[&str] {
        &["vert", "frag", "comp"]
    }
}

pub struct ModelLoader;
impl AssetLoader for ModelLoader {
    fn asset_type(&self) -> AssetType {
        ModelAsset::asset_type()
    }

    fn load(&self, path: &Path) -> Result<Box<dyn Any + Send + Sync>, Error> {
        let asset = ModelAsset::new(path.to_owned());
        Ok(Box::new(asset))
    }

    fn extensions(&self) -> &[&str] {
        &["obj", "fbx", "gltf"]
    }
}
