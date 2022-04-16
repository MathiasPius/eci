use eci_core::backend::{Format, SerializedComponent};
use eci_core::Component;
use serde::Serialize;

pub trait Inserter {
    fn insert<F: Format>(self) -> Vec<SerializedComponent<F>>;
}

macro_rules! impl_inserter{
    ($($v:ident: $T:ident),+) => {
        impl<$($T: Component + Serialize),+> Inserter for ($($T,)+) {
            fn insert<F: Format>(self) -> Vec<SerializedComponent<F>> {
                let ($($v,)+) = self;

                vec![
                    $(
                        SerializedComponent {
                            contents: F::serialize($v).unwrap(),
                            name: $T::COMPONENT_TYPE.to_string(),
                        },
                    )+
                ]
            }
        }
    }
}

macro_rules! impl_all_inserter {
    ($v:ident: $t:ident) => {
        impl_inserter!($v: $t);
    };
    ($vh:ident: $th:ident, $($vr:ident: $tr:ident),*) => {
        impl_inserter!($vh: $th, $($vr: $tr),+);
        impl_all_inserter!($($vr: $tr),+);
    };
}

impl_all_inserter!(
    t1: T1,
    t2: T2,
    t3: T3,
    t4: T4,
    t5: T5,
    t6: T6,
    t7: T7,
    t8: T8,
    t9: T9,
    t10: T10,
    t11: T11,
    t12: T12,
    t13: T13,
    t14: T14,
    t15: T15,
    t16: T16
);
