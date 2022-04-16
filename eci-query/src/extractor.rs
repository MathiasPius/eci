use crate::LockableComponent;
use eci_core::backend::{
    AccessError, ExtractionDescriptor, Format, LockDescriptor, SerializedComponent,
};
use eci_core::Component;

/// Implements mapping from tuples of immutable/mutable references to an
/// extraction descriptor which can be passed to a backend to retrieve
/// and lock components.
pub trait Extractor {
    type Owned;
    fn describe() -> Vec<LockDescriptor>;
    fn extract() -> Vec<ExtractionDescriptor>;

    fn from<F: Format>(
        serialized: Vec<Option<SerializedComponent<F>>>,
    ) -> Result<Option<Self::Owned>, AccessError>;
}

macro_rules! impl_extractor {
    ($head:ident) => {
        impl<$head> Extractor for $head where
            $head: LockableComponent,
        {
            type Owned = $head::Inner;

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

            fn from<F: Format>(serialized: Vec<Option<SerializedComponent<F>>>) -> Result<Option<Self::Owned>, AccessError> {
                Ok(<$head as LockableComponent>::deserialize(serialized.into_iter().next().unwrap())?)
            }
        }
    };
    ($head:ident, $($rest:ident),* ) => {
        impl<$head, $( $rest ),*> Extractor for ($head, $( $rest ),*) where
            $head: LockableComponent,
            $( $rest: LockableComponent),*
        {
            type Owned = ($head::Inner, $( $rest::Inner ),*);

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

            fn from<F: Format>(serialized: Vec<Option<SerializedComponent<F>>>) -> Result<Option<Self::Owned>, AccessError> {
                let mut iter = serialized.into_iter();
                Ok(Some((
                    if let Some(inner) = <$head as LockableComponent>::deserialize(iter.next().unwrap())? {
                        inner
                    } else {
                        return Ok(None)
                    },
                    $(
                    if let Some(inner) = <$rest as LockableComponent>::deserialize(iter.next().unwrap())? {
                        inner
                    } else {
                        return Ok(None)
                    } ),*
                )))
            }
        }

        impl_extractor!( $( $rest ),* );
    };
}

impl_extractor!(T1, T2, T3, T4, T5, T6, T8, T9, T10, T11, T12, T13, T14, T15, T16);
