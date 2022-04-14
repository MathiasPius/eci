use std::{error::Error, fmt::Display};

use serde::{de::DeserializeOwned, Serialize};

use crate::Entity;

#[derive(Debug)]
pub enum AccessError {
    Implementation(Box<dyn Error>),
    Serialization(Box<dyn Error>),
    Conflict(Entity, String),
}

impl Display for AccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessError::Implementation(inner) => {
                write!(f, "error during access: {}", inner)
            }
            AccessError::Serialization(inner) => {
                write!(f, "error during serialization: {}", inner)
            }
            AccessError::Conflict(entity, component) => {
                write!(f, "failed to insert {component} into {entity}'s table")
            }
        }
    }
}

impl Error for AccessError {}

impl AccessError {
    pub fn implementation<T: Error + 'static>(err: T) -> Self {
        AccessError::Implementation(Box::new(err))
    }

    pub fn serialization<T: Error + 'static>(err: T) -> Self {
        AccessError::Serialization(Box::new(err))
    }
}

pub trait AccessBackend<F: Format> {
    fn write_components(
        &self,
        entity: Entity,
        components: Vec<SerializedComponent<F>>,
    ) -> Result<(), AccessError>;

    fn read_components(
        &self,
        entity: Entity,
        descriptors: Vec<ExtractionDescriptor>,
    ) -> Result<Vec<SerializedComponent<F>>, AccessError>;
}

pub trait Format: Display {
    type Data: Into<Vec<u8>> + From<Vec<u8>>;
    fn serialize<T: Serialize>(value: T) -> Result<Self::Data, AccessError>;
    fn deserialize<T: DeserializeOwned>(value: &Self::Data) -> Result<T, AccessError>;
}

pub struct SerializedComponent<F: Format> {
    pub contents: F::Data,
    pub name: String,
}

pub struct ExtractionDescriptor {
    pub name: String,
}

/*
pub trait ToSerializedComponent<F: Format> {
    fn to_serialized_components(self) -> Result<Vec<SerializedComponent<F>>, AccessError>;
}

impl<F: Format, T: Component + Serialize> ToSerializedComponent<F> for T {
    fn to_serialized_components(self) -> Result<Vec<SerializedComponent<F>>, AccessError> {
        Ok(vec![SerializedComponent {
            name: T::NAME,
            version: T::VERSION,
            contents: F::serialize(&self)?,
        }])
    }
}

impl<F, A, B> ToSerializedComponent<F> for (A, B)
where
    F: Format,
    A: Component + Serialize,
    B: Component + Serialize,
{
    fn to_serialized_components(self) -> Result<Vec<SerializedComponent<F>>, AccessError> {
        let mut first = self.0.to_serialized_components()?;
        first.extend(self.1.to_serialized_components()?);
        Ok(first)
    }
}

impl<F, A, B, C> ToSerializedComponent<F> for (A, B, C)
where
    F: Format,
    A: Component + Serialize,
    B: Component + Serialize,
    C: Component + Serialize,
{
    fn to_serialized_components(self) -> Result<Vec<SerializedComponent<F>>, AccessError> {
        let mut first = (self.0, self.1).to_serialized_components()?;
        first.extend(self.2.to_serialized_components()?);
        Ok(first)
    }
}

impl<F, A, B, C, D> ToSerializedComponent<F> for (A, B, C, D)
where
    F: Format,
    A: Component + Serialize,
    B: Component + Serialize,
    C: Component + Serialize,
    D: Component + Serialize,
{
    fn to_serialized_components(self) -> Result<Vec<SerializedComponent<F>>, AccessError> {
        let mut first = (self.0, self.1, self.2).to_serialized_components()?;
        first.extend(self.3.to_serialized_components()?);
        Ok(first)
    }
}

impl<F, A, B, C, D, E> ToSerializedComponent<F> for (A, B, C, D, E)
where
    F: Format,
    A: Component + Serialize,
    B: Component + Serialize,
    C: Component + Serialize,
    D: Component + Serialize,
    E: Component + Serialize,
{
    fn to_serialized_components(self) -> Result<Vec<SerializedComponent<F>>, AccessError> {
        let mut first = (self.0, self.1, self.2, self.3).to_serialized_components()?;
        first.extend(self.4.to_serialized_components()?);
        Ok(first)
    }
}

pub enum ExtractionDescriptor {
    Entity,
    Component(ComponentExtractionDescriptor),
}

pub struct ComponentExtractionDescriptor {
    pub name: &'static str,
    pub version: Version,
}

pub trait FromSerializedComponent<F: Format>: Sized {
    fn from_serialized_components(
        entity: Entity,
        component: &[SerializedComponent<F>],
    ) -> Result<Self, AccessError>;
    fn to_component_descriptor() -> Vec<ExtractionDescriptor>;
}

impl<F, T> FromSerializedComponent<F> for T
where
    F: Format,
    T: Component + DeserializeOwned,
{
    fn from_serialized_components(
        _entity: Entity,
        component: &[SerializedComponent<F>],
    ) -> Result<Self, AccessError> {
        F::deserialize(&component[0].contents).map_err(AccessError::serialization)
    }

    fn to_component_descriptor() -> Vec<ExtractionDescriptor> {
        vec![ExtractionDescriptor::Component(
            ComponentExtractionDescriptor {
                name: T::NAME,
                version: T::VERSION,
            },
        )]
    }
}

impl<F: Format> FromSerializedComponent<F> for Entity {
    fn from_serialized_components(
        entity: Entity,
        _component: &[SerializedComponent<F>],
    ) -> Result<Self, AccessError> {
        Ok(entity)
    }

    fn to_component_descriptor() -> Vec<ExtractionDescriptor> {
        vec![ExtractionDescriptor::Entity]
    }
}

impl<F, A, B> FromSerializedComponent<F> for (A, B)
where
    F: Format,
    A: FromSerializedComponent<F>,
    B: FromSerializedComponent<F>,
{
    fn from_serialized_components(
        entity: Entity,
        component: &[SerializedComponent<F>],
    ) -> Result<Self, AccessError> {
        Ok((
            A::from_serialized_components(entity, &component[0..1])?,
            B::from_serialized_components(entity, &component[1..2])?,
        ))
    }

    fn to_component_descriptor() -> Vec<ExtractionDescriptor> {
        let mut first = A::to_component_descriptor();
        first.extend(B::to_component_descriptor());
        first
    }
}

impl<F, A, B, C> FromSerializedComponent<F> for (A, B, C)
where
    F: Format,
    A: FromSerializedComponent<F>,
    B: FromSerializedComponent<F>,
    C: FromSerializedComponent<F>,
{
    fn from_serialized_components(
        entity: Entity,
        component: &[SerializedComponent<F>],
    ) -> Result<Self, AccessError> {
        Ok((
            A::from_serialized_components(entity, &component[0..1])?,
            B::from_serialized_components(entity, &component[1..2])?,
            C::from_serialized_components(entity, &component[2..3])?,
        ))
    }

    fn to_component_descriptor() -> Vec<ExtractionDescriptor> {
        let mut first = <(A, B)>::to_component_descriptor();
        first.extend(C::to_component_descriptor());
        first
    }
}
 */
