use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    backend::{ComponentStorage, StorageBackend},
    component::Component,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Entity(Uuid);

impl Entity {
    pub fn new() -> Entity {
        Entity(Uuid::new_v4())
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct World<B>
where
    B: StorageBackend,
{
    backend: B,
}

impl<B> World<B>
where
    B: StorageBackend,
{
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn get<T: Component>(&self, entity: Entity) -> ComponentStorage<T> {
        self.backend.get(entity)
    }

    pub fn spawn(&mut self) -> EntityBuilder<'_, B> {
        EntityBuilder {
            entity: self.backend.spawn(),
            world: self,
        }
    }

    pub fn insert<T: Component>(&mut self, entity: Entity, component: T) -> ComponentStorage<T> {
        self.backend.insert(entity, component)
    }

    pub fn remove<T: Component>(&mut self, entity: Entity) -> T {
        self.backend.remove(entity)
    }
}

pub struct EntityBuilder<'world, B: StorageBackend> {
    entity: Entity,
    world: &'world mut World<B>,
}

impl<'world, B: StorageBackend> EntityBuilder<'world, B> {
    pub fn insert<T: Component>(self, component: T) -> Self {
        self.world.insert(self.entity, component);
        self
    }

    pub fn id(self) -> Entity {
        self.entity
    }
}
