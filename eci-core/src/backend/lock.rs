use std::{error::Error, fmt::Display};

use uuid::Uuid;

use crate::Entity;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LockingMode {
    Read,
    Write,
}

impl Display for LockingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockingMode::Read => write!(f, "read"),
            LockingMode::Write => write!(f, "write"),
        }
    }
}

#[derive(Debug)]
pub enum LockingError {
    Implementation(Box<dyn Error>),
    Conflict(Entity, String, LockingMode),
}

impl Display for LockingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockingError::Implementation(inner) => {
                write!(f, "error while acquiring lock: {}", inner)
            }
            LockingError::Conflict(entity, component, mode) => write!(
                f,
                "conflicting lock for {entity}'s {component} while acquiring {mode} lock"
            ),
        }
    }
}

impl Error for LockingError {}

impl LockingError {
    pub fn implementation<T: Error + 'static>(err: T) -> Self {
        LockingError::Implementation(Box::new(err))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Lock(Uuid);

impl Lock {
    pub fn new() -> Lock {
        Lock(Uuid::new_v4())
    }

    pub fn id(&self) -> String {
        self.0.to_string()
    }
}

impl Display for Lock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait LockingBackend {
    fn acquire_lock(
        &self,
        entity: Entity,
        descriptors: Vec<LockDescriptor>,
        expires_in: std::time::Duration,
    ) -> Result<Lock, LockingError>;
    fn release_lock(&self, lock: Lock) -> Result<(), LockingError>;
}

pub struct LockDescriptor {
    pub mode: LockingMode,
    pub name: String,
}
