use eci_core::{
    backend::{
        AccessBackend, AccessError, Backend, ExtractionDescriptor, Format, LockDescriptor,
        LockingMode, SerializedComponent,
    },
    Component, Entity,
};

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
    ) -> Result<Self::Owned, AccessError>;
}

macro_rules! impl_extractor {
    ($head:ident) => {
        impl<$head> Extractor for $head where
            $head: LockableComponent
        {
            type Owned = Option<$head::Inner>;

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

            fn from<F: Format>(serialized: Vec<Option<SerializedComponent<F>>>) -> Result<Self::Owned, AccessError> {
                Ok(<$head as LockableComponent>::deserialize(serialized.into_iter().next().unwrap())?)
            }
        }
    };
    ($head:ident, $($rest:ident),* ) => {
        impl<$head, $( $rest ),*> Extractor for ($head, $( $rest ),*) where
            $head: LockableComponent,
            $( $rest: LockableComponent),*
        {
            type Owned = (Option<$head::Inner>, $( Option<$rest::Inner> ),*);

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

            fn from<F: Format>(serialized: Vec<Option<SerializedComponent<F>>>) -> Result<Self::Owned, AccessError> {
                let mut iter = serialized.into_iter();
                Ok((
                    <$head as LockableComponent>::deserialize(iter.next().unwrap())?,
                    $( <$rest as LockableComponent>::deserialize(iter.next().unwrap())? ),*
                ))
            }
        }

        impl_extractor!( $( $rest ),* );
    };
}

impl_extractor!(T1, T2, T3, T4, T5, T6, T8, T9, T10, T11, T12, T13, T14, T15, T16);

trait TransposeOptionTuple {
    type Transposed;
    fn transpose(self) -> Self::Transposed;
}

macro_rules! impl_transpose_option_tuple {
    ($($v:ident: $T:ident),+ $(,)?) => {
        impl<$($T),+> TransposeOptionTuple for ($(Option<$T>,)+) {
            type Transposed = Option<($($T,)+)>;
            fn transpose(self) -> Self::Transposed {
                let ($($v,)+) = self;
                Some(($($v?,)+))
            }
        }
    }
}

macro_rules! impl_all_transpose_tuples {
    ($v:ident: $t:ident) => {
        impl_transpose_option_tuple!($v: $t);
    };
    ($vh:ident: $th:ident, $($vr:ident: $tr:ident),*) => {
        impl_transpose_option_tuple!($vh: $th, $($vr: $tr),+);
        impl_all_transpose_tuples!($($vr: $tr),+);
    };
}

impl_all_transpose_tuples!(
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

trait Inserter {
    fn insert<F: Format>(self) -> Vec<SerializedComponent<F>>;
}

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

trait TypedBackend<F: Format> {
    fn get<Select>(
        &self,
        entity: Entity,
    ) -> Result<<<Select as Extractor>::Owned as TransposeOptionTuple>::Transposed, AccessError>
    where
        Select: Extractor,
        Select::Owned: TransposeOptionTuple;

    fn put<T>(&self, entity: Entity, components: T) -> Result<(), AccessError>
    where
        T: Inserter;
}

impl<F: Format> TypedBackend<F> for Backend<F> {
    fn get<Select>(
        &self,
        entity: Entity,
    ) -> Result<<<Select as Extractor>::Owned as TransposeOptionTuple>::Transposed, AccessError>
    where
        Select: Extractor,
        Select::Owned: TransposeOptionTuple,
    {
        let components = Select::from(self.read_components(entity, Select::extract())?)?;

        Ok(components.transpose())
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

        let components = backend.get::<(&mut CounterA, &CounterA)>(entity).unwrap();

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
        assert_eq!(
            backend
                .get::<(&CounterA, &StringComponent)>(a)
                .unwrap()
                .unwrap(),
            (CounterA(10), StringComponent("Hello".to_string()))
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
                .unwrap(),
            (CounterA(30), StringComponent("Hello".to_string()))
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
                .unwrap(),
            (CounterA(1), CounterC(3), CounterB(2))
        );
    }
}
