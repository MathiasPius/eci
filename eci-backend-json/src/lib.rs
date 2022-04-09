use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use eci_core::{
    backend::{ComponentStorage, Format, SerializeableBackend, StorageBackend},
    Component, Entity, Version,
};

#[derive(Debug, Serialize, Deserialize)]
struct InternalJsonComponent {
    name: String,
    version: Version,
    inner: serde_json::Value,
}

fn find_component<T: Component>(component: &InternalJsonComponent) -> bool {
    component.name == T::NAME && component.version == T::VERSION
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct InternalJsonEntity {
    components: Vec<InternalJsonComponent>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct InternalJsonState {
    pub entities: HashMap<Entity, InternalJsonEntity>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct JsonBackend {
    state: InternalJsonState,
}

impl StorageBackend for JsonBackend {
    fn update<T: Component>(&mut self, component: ComponentStorage<T>) -> T {
        let serialized_component = serde_json::to_value(component.component).unwrap();

        let old_component = self
            .state
            .entities
            .get_mut(&component.entity)
            .unwrap()
            .components
            .iter_mut()
            .find(|c| find_component::<T>(c))
            .map(|internal_component| {
                std::mem::replace(&mut internal_component.inner, serialized_component)
            })
            .unwrap();

        serde_json::from_value(old_component).unwrap()
    }

    fn get<T: Component>(&self, entity: Entity) -> ComponentStorage<T> {
        let component = self
            .state
            .entities
            .get(&entity)
            .unwrap()
            .components
            .iter()
            .find(|c| find_component::<T>(c))
            .unwrap();

        ComponentStorage {
            entity,
            component: serde_json::from_value(component.inner.clone()).unwrap(),
        }
    }

    fn spawn(&mut self) -> Entity {
        let entity = Entity::new();
        self.state
            .entities
            .insert(entity, InternalJsonEntity::default());

        entity
    }

    fn insert<T: Component>(&mut self, entity: Entity, component: T) -> ComponentStorage<T> {
        let components = &mut self.state.entities.get_mut(&entity).unwrap().components;

        assert!(components
            .iter_mut()
            .find(|c| find_component::<T>(c))
            .is_none());

        components.push(InternalJsonComponent {
            name: T::NAME.to_string(),
            version: T::VERSION,
            inner: serde_json::to_value(&component).unwrap(),
        });

        ComponentStorage { entity, component }
    }

    fn remove<T: Component>(&mut self, entity: Entity) -> T {
        let entity = self.state.entities.get_mut(&entity).unwrap();

        let index = entity
            .components
            .iter()
            .position(|c| find_component::<T>(c))
            .unwrap();
        serde_json::from_value(entity.components.remove(index).inner).unwrap()
    }
}

pub struct JsonString(String);

impl From<Vec<u8>> for JsonString {
    fn from(bytes: Vec<u8>) -> Self {
        JsonString(String::from_utf8_lossy(&bytes).to_string())
    }
}

impl Into<Vec<u8>> for JsonString {
    fn into(self) -> Vec<u8> {
        self.0.into_bytes()
    }
}

#[derive(Debug)]
pub struct Json;

impl Format for Json {
    type Type = JsonString;
    type SerializationError = serde_json::Error;
    type DeserializationError = serde_json::Error;
}

impl SerializeableBackend<Json> for JsonBackend {
    fn load(value: <Json as Format>::Type) -> Result<Self, <Json as Format>::DeserializationError> {
        serde_json::from_str(&value.0)
    }

    fn save(&self) -> Result<<Json as Format>::Type, <Json as Format>::SerializationError> {
        Ok(JsonString(serde_json::to_string(self)?))
    }
}

#[cfg(test)]
mod tests {
    use crate::JsonBackend;
    use eci_core::{component::DebugString, World};

    #[test]
    fn test_json_storage() {
        let mut world = World::new(JsonBackend::default());

        let entity = world
            .spawn()
            .insert(DebugString {
                content: "example.org".to_string(),
            })
            .id();

        println!("{}", entity);

        println!("{:#?}", world);
    }
}
