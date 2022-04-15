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
    ) -> Result<Vec<Option<SerializedComponent<F>>>, AccessError>;
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