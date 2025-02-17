use std::any::{Any, TypeId};
use std::collections::HashMap;

pub trait Component: Any + Send + Sync {
    fn component_type() -> TypeId
    where
        Self: 'static,
    {
        TypeId::of::<Self>()
    }
}

pub struct ComponentStorage {
    components: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn add<T: Component>(&mut self, component: T) {
        self.components
            .insert(T::component_type(), Box::new(component));
    }

    pub fn get<T: Component>(&self) -> Option<&T> {
        self.components
            .get(&T::component_type())
            .and_then(|c| c.downcast_ref())
    }

    pub fn get_mut<T: Component>(&mut self) -> Option<&mut T> {
        self.components
            .get_mut(&T::component_type())
            .and_then(|c| c.downcast_mut())
    }

    pub fn remove<T: Component>(&mut self) -> Option<Box<T>> {
        self.components
            .remove(&T::component_type())
            .map(|c| c.downcast().ok())
            .flatten()
    }

    pub fn has<T: Component>(&self) -> bool {
        self.components.contains_key(&T::component_type())
    }

    pub fn has_type(&self, type_id: TypeId) -> bool {
        self.components.contains_key(&type_id)
    }
}

impl Default for ComponentStorage {
    fn default() -> Self {
        Self::new()
    }
}

// Common components
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Component for Transform {}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<crate::graphics::MeshVertex>,
    pub indices: Vec<u32>,
}

impl Component for Mesh {}

#[derive(Debug, Clone, Default)]
pub struct RigidBody {
    pub velocity: glam::Vec3,
    pub mass: f32,
}

impl Component for RigidBody {}
