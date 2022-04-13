use std::fmt::Display;

use eci_core::backend::{AccessError, Format};
use serde::{de::DeserializeOwned, Serialize};

pub struct Json;

impl Format for Json {
    type Data = Vec<u8>;

    fn serialize<T: Serialize>(value: T) -> Result<Self::Data, AccessError> {
        Ok(serde_json::to_string(&value)
            .map_err(AccessError::serialization)?
            .into())
    }

    fn deserialize<T: DeserializeOwned>(value: &Self::Data) -> Result<T, AccessError> {
        let source = String::from_utf8(value.to_vec()).map_err(AccessError::serialization)?;
        serde_json::from_str(&source).map_err(AccessError::serialization)
    }
}

impl Display for Json {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "json")
    }
}
