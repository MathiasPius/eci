use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::{component::Component, Entity};

#[derive(Debug)]
pub struct ComponentStorage<T: Component> {
    pub entity: Entity,
    pub component: T,
}

impl<T> Deref for ComponentStorage<T>
where
    T: Component,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl<T> DerefMut for ComponentStorage<T>
where
    T: Component,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

pub trait StorageBackend {
    fn spawn(&mut self) -> Entity;
    fn update<T: Component>(&mut self, component: ComponentStorage<T>) -> T;
    fn insert<T: Component>(&mut self, entity: Entity, component: T) -> ComponentStorage<T>;
    fn remove<T: Component>(&mut self, entity: Entity) -> T;
    fn get<T: Component>(&self, entity: Entity) -> ComponentStorage<T>;
}

pub trait Format: Debug {
    type Type;
    type SerializationError: Debug;
    type DeserializationError: Debug;
}

pub trait SerializeableBackend<F: Format>: Sized {
    fn load(value: F::Type) -> Result<Self, F::DeserializationError>;
    fn save(&self) -> Result<F::Type, F::SerializationError>;
}
