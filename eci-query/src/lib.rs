use eci_core::{
    backend::{
        AccessBackend, AccessError, Backend, ExtractionDescriptor, Format, LockDescriptor,
        LockingMode, SerializedComponent,
    },
    Component, Entity,
};

use serde::de::DeserializeOwned;

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
            $head: LockableComponent,
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

            fn from<F: Format>(mut serialized: Vec<Option<SerializedComponent<F>>>) -> Result<Self::Owned, AccessError> {
                Ok(<$head as LockableComponent>::deserialize(serialized.pop().unwrap())?)
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

            fn from<F: Format>(mut serialized: Vec<Option<SerializedComponent<F>>>) -> Result<Self::Owned, AccessError> {
                Ok((
                    <$head as LockableComponent>::deserialize(serialized.pop().unwrap())?,
                    $( <$rest as LockableComponent>::deserialize(serialized.pop().unwrap())? ),*
                ))
            }
        }

        impl_extractor!( $( $rest ),* );
    };
}

impl_extractor!(T1, T2, T3, T4, T5, T6, T8, T9, T10, T11, T12, T13, T14, T15, T16);

trait TypedBackend<F: Format> {
    fn get<Select: Extractor>(&self, entity: Entity) -> Result<Select::Owned, AccessError>;
}

impl<F: Format> TypedBackend<F> for Backend<F> {
    fn get<Select: Extractor>(&self, entity: Entity) -> Result<Select::Owned, AccessError> {
        Select::from(self.read_components(entity, Select::extract())?)
    }
}

#[cfg(test)]
mod tests {
    use eci_backend_sqlite::SqliteBackend;
    use eci_core::{backend::Backend, Component, Entity};
    use eci_format_json::Json;
    use serde::Deserialize;

    use crate::TypedBackend;

    #[derive(Debug, Component, Deserialize)]
    struct SomeComponent {}

    #[test]
    fn it_works() {
        let backend = Backend::<Json>::from_joint(SqliteBackend::in_memory().unwrap());

        let entity = Entity::new();

        let components = backend.get::<&SomeComponent>(entity).unwrap();

        println!("{:?}", components);
    }
}
