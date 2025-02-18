use super::asset::{Asset, AssetType, ModelAsset, ShaderAsset, TextureAsset};
use crate::core::error::Error;
use std::any::Any;
use std::path::Path;

#[derive(Clone)]
pub enum AssetLoader {
    Texture(TextureLoader),
    Shader(ShaderLoader),
    Model(ModelLoader),
}

impl AssetLoader {
    pub fn asset_type(&self) -> AssetType {
        match self {
            AssetLoader::Texture(loader) => loader.asset_type(),
            AssetLoader::Shader(loader) => loader.asset_type(),
            AssetLoader::Model(loader) => loader.asset_type(),
        }
    }

    pub fn load_typed<T: Asset + 'static>(&self, path: &Path) -> Result<T, Error> {
        if self.asset_type() != T::asset_type() {
            return Err(Error::ResourceError(format!(
                "Loader for {:?} cannot load asset of type {:?}",
                self.asset_type(),
                T::asset_type()
            )));
        }

        let boxed = match self {
            AssetLoader::Texture(loader) => loader.load(path)?,
            AssetLoader::Shader(loader) => loader.load(path)?,
            AssetLoader::Model(loader) => loader.load(path)?,
        };

        boxed.downcast::<T>().map(|b| *b).map_err(|_| {
            Error::ResourceError(format!(
                "Failed to convert loaded asset to requested type {}",
                std::any::type_name::<T>()
            ))
        })
    }

    pub fn extensions(&self) -> &[&str] {
        match self {
            AssetLoader::Texture(loader) => loader.extensions(),
            AssetLoader::Shader(loader) => loader.extensions(),
            AssetLoader::Model(loader) => loader.extensions(),
        }
    }
}

#[derive(Clone)]
pub struct TextureLoader;
impl TextureLoader {
    pub fn asset_type(&self) -> AssetType {
        TextureAsset::asset_type()
    }

    pub fn load(&self, path: &Path) -> Result<Box<dyn Any + Send + Sync>, Error> {
        let asset = TextureAsset::new(path.to_owned());
        Ok(Box::new(asset))
    }

    pub fn extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg"]
    }
}

#[derive(Clone)]
pub struct ShaderLoader;
impl ShaderLoader {
    pub fn asset_type(&self) -> AssetType {
        ShaderAsset::asset_type()
    }

    pub fn load(&self, path: &Path) -> Result<Box<dyn Any + Send + Sync>, Error> {
        let asset = ShaderAsset::new(path.to_owned());
        Ok(Box::new(asset))
    }

    pub fn extensions(&self) -> &[&str] {
        &["vert", "frag", "comp"]
    }
}

#[derive(Clone)]
pub struct ModelLoader;
impl ModelLoader {
    pub fn asset_type(&self) -> AssetType {
        ModelAsset::asset_type()
    }

    pub fn load(&self, path: &Path) -> Result<Box<dyn Any + Send + Sync>, Error> {
        let asset = ModelAsset::new(path.to_owned());
        Ok(Box::new(asset))
    }

    pub fn extensions(&self) -> &[&str] {
        &["obj", "fbx", "gltf"]
    }
}
