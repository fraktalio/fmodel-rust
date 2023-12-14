use crate::saga::Saga;
use crate::Sum;

/// Combine two sagas into one.
/// Creates a new instance of a Saga by combining two sagas of type `AR1`, `A1` and `AR2`, `A2` into a new saga of type `Sum<AR1, AR2>`, `Sum<A1, A2>`
/// We encourage you to create your own combine functions to combine more sagas (three, four, five, ...) into one.
pub fn combine<'a, AR1, A1, AR2, A2>(
    saga1: Saga<'a, AR2, A1>,
    saga2: Saga<'a, AR1, A2>,
) -> Saga<'a, Sum<AR1, AR2>, Sum<A1, A2>> {
    let new_react = Box::new(move |ar: &Sum<AR1, AR2>| match ar {
        Sum::First(ar1) => {
            let a2 = (saga2.react)(ar1);
            a2.into_iter().map(|a: A2| Sum::Second(a)).collect()
        }
        Sum::Second(ar2) => {
            let a1 = (saga1.react)(ar2);
            a1.into_iter().map(|a: A1| Sum::First(a)).collect()
        }
    });

    Saga { react: new_react }
}
