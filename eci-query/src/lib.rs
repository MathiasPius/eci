use std::fmt::Debug;

use eci_core::{
    backend::{
        AccessBackend, AccessError, Backend, BackendError, ExtractionDescriptor, Format, Lock,
        LockDescriptor, LockingBackend, LockingError, LockingMode, SerializedComponent,
    },
    Component, Entity,
};

use log::debug;
use serde::{de::DeserializeOwned, Serialize};

trait LockableComponent {
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

trait Extractor {
    type Owned;
    fn describe() -> Vec<LockDescriptor>;
    fn extract() -> Vec<ExtractionDescriptor>;

    fn from<F: Format>(
        serialized: Vec<Option<SerializedComponent<F>>>,
    ) -> Result<Option<Self::Owned>, AccessError>;
}

macro_rules! impl_extractor {
    ($head:ident) => {
        impl<$head> Extractor for $head where
            $head: LockableComponent,
        {
            type Owned = $head::Inner;

            fn describe() -> Vec<LockDescriptor> {
                vec![
                    $head::as_lock(),
                ]
            }

            fn extract() -> Vec<ExtractionDescriptor> {
                vec![
                    ExtractionDescriptor { name: $head::Inner::COMPONENT_TYPE.to_string() },
                ]
            }

            fn from<F: Format>(serialized: Vec<Option<SerializedComponent<F>>>) -> Result<Option<Self::Owned>, AccessError> {
                Ok(<$head as LockableComponent>::deserialize(serialized.into_iter().next().unwrap())?)
            }
        }
    };
    ($head:ident, $($rest:ident),* ) => {
        impl<$head, $( $rest ),*> Extractor for ($head, $( $rest ),*) where
            $head: LockableComponent,
            $( $rest: LockableComponent),*
        {
            type Owned = ($head::Inner, $( $rest::Inner ),*);

            fn describe() -> Vec<LockDescriptor> {
                vec![
                    $head::as_lock(),
                    $( $rest::as_lock() ),*
                ]
            }

            fn extract() -> Vec<ExtractionDescriptor> {
                vec![
                    ExtractionDescriptor { name: $head::Inner::COMPONENT_TYPE.to_string() },
                    $( ExtractionDescriptor { name: $rest::Inner::COMPONENT_TYPE.to_string() } ),*
                ]
            }

            fn from<F: Format>(serialized: Vec<Option<SerializedComponent<F>>>) -> Result<Option<Self::Owned>, AccessError> {
                let mut iter = serialized.into_iter();
                Ok(Some((
                    if let Some(inner) = <$head as LockableComponent>::deserialize(iter.next().unwrap())? {
                        inner
                    } else {
                        return Ok(None)
                    },
                    $(
                    if let Some(inner) = <$rest as LockableComponent>::deserialize(iter.next().unwrap())? {
                        inner
                    } else {
                        return Ok(None)
                    } ),*
                )))
            }
        }

        impl_extractor!( $( $rest ),* );
    };
}

impl_extractor!(T1, T2, T3, T4, T5, T6, T8, T9, T10, T11, T12, T13, T14, T15, T16);

trait Inserter {
    fn insert<F: Format>(self) -> Vec<SerializedComponent<F>>;
}

trait RefCast<'borrow, 'owned: 'borrow>: 'borrow {
    type Owned;
    fn refcast(owned: &'owned mut Self::Owned) -> Self;
}

impl<'borrow, 'owned: 'borrow, A> RefCast<'borrow, 'owned> for &'borrow A {
    type Owned = A;

    fn refcast(a: &'owned mut Self::Owned) -> Self {
        &*a
    }
}

impl<'borrow, 'owned: 'borrow, A> RefCast<'borrow, 'owned> for &'borrow mut A {
    type Owned = A;

    fn refcast(a: &'owned mut Self::Owned) -> Self {
        a
    }
}

macro_rules! borrow_tuple {
    ($vh:ident: $th:ident : $ih:ident) => {
        impl<'borrow, 'owned: 'borrow, $th, $ih> RefCast<'borrow, 'owned> for ($th,) where
            $th: RefCast<'borrow, 'owned, Owned = $ih> + 'borrow {
            type Owned = ($ih,);

            fn refcast((ref mut $vh,): &'owned mut ($th::Owned,)) -> Self {
                ($th::refcast($vh),)
            }
        }
    };

    (  $vh:ident: $th:ident : $ih:ident, $($v:ident: $t:ident : $i:ident),+) => {
        impl<'borrow, 'owned: 'borrow, $th, $( $t ),*, $ih, $( $i ),*> RefCast<'borrow, 'owned> for ($th, $($t),*) where
            $th: RefCast<'borrow, 'owned, Owned = $ih> + 'borrow,
            $( $t: RefCast<'borrow, 'owned, Owned = $i> + 'borrow ),* {
            type Owned = ($ih, $( $i ),*);

            fn refcast( (ref mut $vh, ref mut $( $v ),*) : &'owned mut ($th::Owned, $( $t::Owned),* )) -> Self {
                (
                    ($th::refcast($vh), $( $t::refcast($v) ),*)
                )
            }
        }

        borrow_tuple!($( $v : $t : $i ),*);
    };
}

borrow_tuple!(
    t1: T1: I1,
    t2: T2: I2,
    t3: T3: I3,
    t4: T4: I4,
    t5: T5: I5,
    t6: T6: I6,
    t7: T7: I7,
    t8: T8: I8,
    t9: T9: I9,
    t10: T10: I10,
    t11: T11: I11,
    t12: T12: I12,
    t13: T13: I13,
    t14: T14: I14,
    t15: T15: I15,
    t16: T16: I16
);

macro_rules! impl_inserter{
    ($($v:ident: $T:ident),+) => {
        impl<$($T: Component + Serialize),+> Inserter for ($($T,)+) {
            fn insert<F: Format>(self) -> Vec<SerializedComponent<F>> {
                let ($($v,)+) = self;

                vec![
                    $(
                        SerializedComponent {
                            contents: F::serialize($v).unwrap(),
                            name: $T::COMPONENT_TYPE.to_string(),
                        },
                    )+
                ]
            }
        }
    }
}

macro_rules! impl_all_inserter {
    ($v:ident: $t:ident) => {
        impl_inserter!($v: $t);
    };
    ($vh:ident: $th:ident, $($vr:ident: $tr:ident),*) => {
        impl_inserter!($vh: $th, $($vr: $tr),+);
        impl_all_inserter!($($vr: $tr),+);
    };
}

impl_all_inserter!(
    t1: T1,
    t2: T2,
    t3: T3,
    t4: T4,
    t5: T5,
    t6: T6,
    t7: T7,
    t8: T8,
    t9: T9,
    t10: T10,
    t11: T11,
    t12: T12,
    t13: T13,
    t14: T14,
    t15: T15,
    t16: T16
);

pub struct DropLock {
    lock: Option<Lock>,
    backend: Box<dyn LockingBackend>,
}

impl DropLock {
    pub fn unlock(mut self) -> Result<(), LockingError> {
        if let Some(lock) = self.lock.take() {
            self.backend.release_lock(lock)
        } else {
            Ok(())
        }
    }
}

impl Drop for DropLock {
    fn drop(&mut self) {
        if let Some(lock) = self.lock.take() {
            println!("dropped lock: {lock}");
            self.backend.release_lock(lock).ok();
        }
    }
}

impl Debug for DropLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DropLock")
            .field("lock", &self.lock)
            .finish()
    }
}

struct Locked<T>
where
    T: Extractor,
{
    lock: DropLock,
    inner: <T as Extractor>::Owned,
}

impl<'borrow, 'owned: 'borrow, T> Locked<T>
where
    T: Extractor,
    T: RefCast<'borrow, 'owned, Owned = <T as Extractor>::Owned>,
{
    pub fn unlock(mut self) -> Result<(), LockingError> {
        self.lock.unlock()
    }

    pub fn deref(&'owned mut self) -> T {
        <T as RefCast<'borrow, 'owned>>::refcast(&mut self.inner)
    }
}

impl<T> Debug for Locked<T>
where
    T: Extractor,
    <T as Extractor>::Owned: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Locked")
            .field("lock", &self.lock)
            .field("inner", &self.inner)
            .finish()
    }
}

trait TypedBackend<F: Format> {
    fn get<Select>(&self, entity: Entity) -> Result<Option<Locked<Select>>, BackendError>
    where
        Select: Extractor;

    fn put<T>(&self, entity: Entity, components: T) -> Result<(), AccessError>
    where
        T: Inserter;
}

impl<F: Format> TypedBackend<F> for Backend<F> {
    fn get<Select>(&self, entity: Entity) -> Result<Option<Locked<Select>>, BackendError>
    where
        Select: Extractor,
    {
        let components = Select::from(self.read_components(entity, Select::extract())?)?;

        if let Some(components) = components {
            let lock = self.acquire_lock(
                entity,
                Select::describe(),
                std::time::Duration::from_secs(3600),
            )?;

            Ok(Some(Locked {
                inner: components,
                lock: DropLock {
                    lock: Some(lock),
                    backend: Box::new((*self).clone()),
                },
            }))
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

        assert_eq!(
            lock.deref(),
            (&CounterA(10), &StringComponent("Hello".to_string()))
        );

        assert_eq!(
            lock.deref(),
            (&CounterA(10), &StringComponent("Hello".to_string()))
        );

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
