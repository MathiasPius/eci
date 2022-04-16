pub mod extractor;
pub mod inserter;
pub mod lock;
pub mod refcast;

use eci_core::{
    backend::{
        AccessBackend, AccessError, Backend, BackendError, Format, LockDescriptor, LockingBackend,
        LockingMode, SerializedComponent,
    },
    Component, Entity,
};

use extractor::Extractor;
use inserter::Inserter;
use lock::{DropLock, Locked};
use refcast::RefCast;
use serde::de::DeserializeOwned;

pub trait LockableComponent {
    type Inner: Component + DeserializeOwned;
    fn as_lock() -> LockDescriptor;
    fn deserialize<F: Format>(
        serialized: Option<SerializedComponent<F>>,
    ) -> Result<Option<Self::Inner>, AccessError>;
}

impl<T> LockableComponent for &T
where
    T: Component + DeserializeOwned,
{
    type Inner = T;
    fn as_lock() -> LockDescriptor {
        LockDescriptor {
            mode: LockingMode::Read,
            name: T::COMPONENT_TYPE.to_string(),
        }
    }

    fn deserialize<F: Format>(
        serialized: Option<SerializedComponent<F>>,
    ) -> Result<Option<Self::Inner>, AccessError> {
        serialized
            .map(|component| {
                let data = F::Data::from(component.contents.into());
                F::deserialize::<Self::Inner>(&data)
            })
            .transpose()
    }
}

impl<T> LockableComponent for &mut T
where
    T: Component + DeserializeOwned,
{
    type Inner = T;
    fn as_lock() -> LockDescriptor {
        LockDescriptor {
            mode: LockingMode::Write,
            name: T::COMPONENT_TYPE.to_string(),
        }
    }

    fn deserialize<F: Format>(
        serialized: Option<SerializedComponent<F>>,
    ) -> Result<Option<Self::Inner>, AccessError> {
        serialized
            .map(|component| {
                let data = F::Data::from(component.contents.into());
                F::deserialize::<Self::Inner>(&data)
            })
            .transpose()
    }
}

pub trait TypedBackend<F: Format> {
    fn get<'a, Select>(&self, entity: Entity) -> Result<Option<Locked<Select>>, BackendError>
    where
        Select: Extractor + RefCast<'a, Owned = <Select as Extractor>::Owned>;

    fn put<T>(&self, entity: Entity, components: T) -> Result<(), AccessError>
    where
        T: Inserter;
}

impl<F: Format> TypedBackend<F> for Backend<F> {
    fn get<'a, Select>(&self, entity: Entity) -> Result<Option<Locked<Select>>, BackendError>
    where
        Select: Extractor + RefCast<'a, Owned = <Select as Extractor>::Owned>,
    {
        let components = Select::from(self.read_components(entity, Select::extract())?)?;

        if let Some(components) = components {
            let lock = self.acquire_lock(
                entity,
                Select::describe(),
                std::time::Duration::from_secs(3600),
            )?;

            Ok(Some(Locked::new(
                DropLock::new(lock, Box::new((*self).clone())),
                components,
            )))
        } else {
            Ok(None)
        }
    }

    fn put<T>(&self, entity: Entity, components: T) -> Result<(), AccessError>
    where
        T: Inserter,
    {
        let serialized = components.insert::<F>();
        self.write_components(entity, serialized)
    }
}

#[cfg(test)]
mod tests {
    use eci_backend_sqlite::SqliteBackend;
    use eci_core::{backend::Backend, Component, Entity};
    use eci_format_json::Json;
    use serde::{Deserialize, Serialize};

    use crate::TypedBackend;

    #[derive(Debug, Component, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
    struct CounterA(pub usize);

    #[derive(Debug, Component, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
    struct CounterB(pub usize);

    #[derive(Debug, Component, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
    struct CounterC(pub usize);

    #[derive(Debug, Component, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
    struct StringComponent(pub String);

    #[test]
    fn it_works() {
        let backend = Backend::<Json>::from_joint(SqliteBackend::memory().unwrap());

        let entity = Entity::new();

        let inserts = (CounterA(0),);

        backend.put(entity, inserts).unwrap();

        let components = backend
            .get::<(&mut CounterA, &CounterA)>(entity)
            .unwrap_err();

        println!("{:?}", components);
    }

    #[test]
    fn insert_component() {
        let backend = Backend::<Json>::from_joint(SqliteBackend::memory().unwrap());

        // Insert separately
        let a = Entity::new();
        backend.put(a, (CounterA(10),)).unwrap();
        backend
            .put(a, (StringComponent("Hello".to_string()),))
            .unwrap();

        // Insert collectively
        let b = Entity::new();
        backend
            .put(b, (CounterA(30), StringComponent("Hello".to_string())))
            .unwrap();
    }

    #[test]
    fn get_components() {
        let backend = Backend::<Json>::from_joint(SqliteBackend::memory().unwrap());

        // Insert separately
        let a = Entity::new();
        backend.put(a, (CounterA(10),)).unwrap();
        backend
            .put(a, (StringComponent("Hello".to_string()),))
            .unwrap();

        {
            let mut lock = backend.get::<&CounterA>(a).unwrap().unwrap();
            println!("locked: {:?}", lock.deref());
        }

        {
            let mut lock = backend.get::<&CounterA>(a).unwrap().unwrap();
            println!("locked: {:?}", lock.deref());
        }

        let mut lock = backend
            .get::<(&CounterA, &StringComponent)>(a)
            .unwrap()
            .unwrap();

        {
            let reference = &mut lock;
            assert_eq!(
                reference.deref(),
                (&CounterA(10), &StringComponent("Hello".to_string()))
            );
        };

        // Insert collectively
        let b = Entity::new();
        backend
            .put(b, (CounterA(30), StringComponent("Hello".to_string())))
            .unwrap();
        assert_eq!(
            backend
                .get::<(&CounterA, &StringComponent)>(b)
                .unwrap()
                .unwrap()
                .deref(),
            (&CounterA(30), &StringComponent("Hello".to_string()))
        );
    }

    #[test]
    fn component_ordering() {
        let backend = Backend::<Json>::from_joint(SqliteBackend::memory().unwrap());

        let a = Entity::new();
        backend.put(a, (CounterA(1),)).unwrap();
        backend.put(a, (CounterB(2),)).unwrap();
        backend.put(a, (CounterC(3),)).unwrap();
        assert_eq!(
            backend
                .get::<(&CounterA, &CounterC, &CounterB)>(a)
                .unwrap()
                .unwrap()
                .deref(),
            (&CounterA(1), &CounterC(3), &CounterB(2))
        );
    }
}
