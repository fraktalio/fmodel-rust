use crate::saga::Saga;
use crate::{Sum, Sum3};

/// Combine two sagas into one.
/// Creates a new instance of a Saga by combining two sagas of type `AR1`, `A1` and `AR2`, `A2` into a new saga of type `Sum<AR1, AR2>`, `Sum<A1, A2>`
pub fn combine<'a, AR1, A1, AR2, A2>(
    saga1: Saga<'a, AR1, A1>,
    saga2: Saga<'a, AR2, A2>,
) -> Saga<'a, Sum<AR1, AR2>, Sum<A1, A2>> {
    let new_react = Box::new(move |ar: &Sum<AR1, AR2>| match ar {
        Sum::First(ar1) => {
            let a1 = (saga1.react)(ar1);
            a1.into_iter().map(|a: A1| Sum::First(a)).collect()
        }
        Sum::Second(ar2) => {
            let a2 = (saga2.react)(ar2);
            a2.into_iter().map(|a: A2| Sum::Second(a)).collect()
        }
    });

    Saga { react: new_react }
}

/// Combine three sagas into one.
/// Creates a new instance of a Saga by combining three sagas of type `AR1`, `A1` ,  `AR2`, `A2`, and `AR3`, `A3` into a new saga of type `Sum3<AR1, AR2, AR3>`, `Sum3<A1, A2, A3>`
pub fn combine3<'a, AR1, A1, AR2, A2, AR3, A3>(
    saga1: Saga<'a, AR1, A1>,
    saga2: Saga<'a, AR2, A2>,
    saga3: Saga<'a, AR3, A3>,
) -> Saga<'a, Sum3<AR1, AR2, AR3>, Sum3<A1, A2, A3>> {
    let new_react = Box::new(move |ar: &Sum3<AR1, AR2, AR3>| match ar {
        Sum3::First(ar1) => {
            let a1 = (saga1.react)(ar1);
            a1.into_iter().map(|a: A1| Sum3::First(a)).collect()
        }
        Sum3::Second(ar2) => {
            let a2 = (saga2.react)(ar2);
            a2.into_iter().map(|a: A2| Sum3::Second(a)).collect()
        }
        Sum3::Third(ar3) => {
            let a3 = (saga3.react)(ar3);
            a3.into_iter().map(|a: A3| Sum3::Third(a)).collect()
        }
    });

    Saga { react: new_react }
}
