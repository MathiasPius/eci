mod access;
mod lock;
use std::{error::Error, fmt::Display, sync::Arc};

pub use access::*;
pub use lock::*;

use crate::Entity;

pub trait JointBackend<F: Format>: AccessBackend<F> + LockingBackend {}

impl<T, F> JointBackend<F> for T
where
    F: Format,
    T: AccessBackend<F> + LockingBackend,
{
}

#[derive(Clone)]
pub enum Backend<F: Format> {
    Disjoint {
        locking: Arc<dyn LockingBackend>,
        access: Arc<dyn AccessBackend<F>>,
    },
    Joint {
        backend: Arc<dyn JointBackend<F>>,
    },
}

impl<F: Format> AccessBackend<F> for Backend<F> {
    fn write_components(
        &self,
        entity: Entity,
        components: Vec<SerializedComponent<F>>,
    ) -> Result<(), AccessError> {
        match self {
            Backend::Disjoint { locking: _, access } => access.write_components(entity, components),
            Backend::Joint { backend } => backend.write_components(entity, components),
        }
    }

    fn read_components(
        &self,
        entity: Entity,
        descriptors: Vec<ExtractionDescriptor>,
    ) -> Result<Vec<Option<SerializedComponent<F>>>, AccessError> {
        match self {
            Backend::Disjoint { locking: _, access } => access.read_components(entity, descriptors),
            Backend::Joint { backend } => backend.read_components(entity, descriptors),
        }
    }
}

impl<F: Format> LockingBackend for Backend<F> {
    fn acquire_lock(
        &self,
        entity: Entity,
        descriptors: Vec<LockDescriptor>,
        expires_in: std::time::Duration,
    ) -> Result<Lock, LockingError> {
        match self {
            Backend::Disjoint { locking, access: _ } => {
                locking.acquire_lock(entity, descriptors, expires_in)
            }
            Backend::Joint { backend } => backend.acquire_lock(entity, descriptors, expires_in),
        }
    }

    fn release_lock(&self, lock: Lock) -> Result<(), LockingError> {
        match self {
            Backend::Disjoint { locking, access: _ } => locking.release_lock(lock),
            Backend::Joint { backend } => backend.release_lock(lock),
        }
    }
}

impl<F: Format> Backend<F> {
    pub fn from_joint<T: JointBackend<F> + 'static>(backend: T) -> Self {
        Backend::Joint {
            backend: Arc::new(backend),
        }
    }

    pub fn from_disjoint<A: AccessBackend<F> + 'static, L: LockingBackend + 'static>(
        access: A,
        locking: L,
    ) -> Self {
        Backend::Disjoint {
            access: Arc::new(access),
            locking: Arc::new(locking),
        }
    }
}

#[derive(Debug)]
pub enum BackendError {
    Access(AccessError),
    Locking(LockingError),
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
            BackendError::Access(access) => write!(f, "access error {}", access),
            BackendError::Locking(locking) => write!(f, "locking error {}", locking),
        }
    }
}

impl Error for BackendError {}
