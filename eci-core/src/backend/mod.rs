mod access;
mod lock;
use std::{error::Error, fmt::Display};

pub use access::*;
pub use lock::*;

#[derive(Debug)]
pub enum BackendError {
    Locking(LockingError),
    Access(AccessError),
}

impl From<LockingError> for BackendError {
    fn from(locking: LockingError) -> Self {
        BackendError::Locking(locking)
    }
}

impl From<AccessError> for BackendError {
    fn from(access: AccessError) -> Self {
        BackendError::Access(access)
    }
}

impl Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendError::Locking(locking) => write!(f, "backend (locking) error: {}", locking),
            BackendError::Access(access) => write!(f, "backend (access) error: {}", access),
        }
    }
}

impl Error for BackendError {}
