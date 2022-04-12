use std::{error::Error, fmt::Display};

use semver::Version;
use serde::{de::DeserializeOwned, Serialize};

use crate::{Component, Entity};

#[derive(Debug)]
pub enum AccessError {
    Implementation(Box<dyn Error>),
    Serialization(Box<dyn Error>),
    Conflict(Entity, &'static str, Version),
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
            AccessError::Conflict(entity, component, version) => {
                write!(
                    f,
                    "failed to insert {component}({version}) into {entity}'s table"
                )
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

pub trait AccessBackend {
    fn write_components<F: Format, T: ToSerializedComponent<F>>(
        &self,
        entity: Entity,
        components: T,
    ) -> Result<(), AccessError>;
}

pub trait Format: Display {
    type Data: Into<Vec<u8>> + From<Vec<u8>>;
    fn serialize<T: Serialize>(value: T) -> Result<Self::Data, AccessError>;
    fn deserialize<T: DeserializeOwned>(value: Self::Data) -> Result<T, AccessError>;
}

pub struct SerializedComponent<F: Format> {
    pub contents: F::Data,
    pub name: &'static str,
    pub version: Version,
}

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
