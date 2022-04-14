mod access;
mod lock;
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

pub enum Backend<F: Format> {
    Disjoint {
        locking: Box<dyn LockingBackend>,
        access: Box<dyn AccessBackend<F>>,
    },
    Joint {
        backend: Box<dyn JointBackend<F>>,
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

impl<F: Format> Backend<F> {
    pub fn from_joint<T: JointBackend<F> + 'static>(backend: T) -> Self {
        Backend::Joint {
            backend: Box::new(backend),
        }
    }

    pub fn from_disjoint<A: AccessBackend<F> + 'static, L: LockingBackend + 'static>(
        access: A,
        locking: L,
    ) -> Self {
        Backend::Disjoint {
            access: Box::new(access),
            locking: Box::new(locking),
        }
    }
}
