pub trait RefCast<'a> {
    type Owned;
    fn refcast(owned: &'a mut Self::Owned) -> Self;
}

impl<'a, A> RefCast<'a> for &'a A {
    type Owned = A;

    fn refcast(a: &'a mut Self::Owned) -> Self {
        &*a
    }
}

impl<'a, A> RefCast<'a> for &'a mut A {
    type Owned = A;

    fn refcast(a: &'a mut Self::Owned) -> Self {
        a
    }
}

macro_rules! borrow_tuple {
    ($vh:ident: $th:ident : $ih:ident) => {
        impl<'a, $th, $ih> RefCast<'a> for ($th,) where
            $th: RefCast<'a, Owned = $ih> + 'a {
            type Owned = ($ih,);

            fn refcast((ref mut $vh,): &'a mut ($th::Owned,)) -> Self {
                ($th::refcast($vh),)
            }
        }
    };

    (  $vh:ident: $th:ident : $ih:ident, $($v:ident: $t:ident : $i:ident),+) => {
        impl<'a, $th, $( $t ),*, $ih, $( $i ),*> RefCast<'a> for ($th, $($t),*) where
            $th: RefCast<'a, Owned = $ih> + 'a,
            $( $t: RefCast<'a, Owned = $i> + 'a ),* {
            type Owned = ($ih, $( $i ),*);

            fn refcast( (ref mut $vh, ref mut $( $v ),*) : &'a mut ($th::Owned, $( $t::Owned),* )) -> Self {
                (
                    ($th::refcast($vh), $( $t::refcast($v) ),*)
                )
            }
        }

        borrow_tuple!($( $v : $t : $i ),*);
    };
}

borrow_tuple!(
    t1: T1: I1,
    t2: T2: I2,
    t3: T3: I3,
    t4: T4: I4,
    t5: T5: I5,
    t6: T6: I6,
    t7: T7: I7,
    t8: T8: I8,
    t9: T9: I9,
    t10: T10: I10,
    t11: T11: I11,
    t12: T12: I12,
    t13: T13: I13,
    t14: T14: I14,
    t15: T15: I15,
    t16: T16: I16
);
