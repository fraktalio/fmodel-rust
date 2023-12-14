use crate::decider::Decider;
use crate::Sum;

/// Combine two deciders into one bigger decider
/// Creates a new instance of a Decider by combining two deciders of type `C1`, `S1`, `E1` and `C2`, `S2`, `E2` into a new decider of type `Sum<C, C2>`, `(S, S2)`, `Sum<E, E2>`
/// We encourage you to create your own combine functions to combine more deciders (three, four, five, ...) into one.
pub fn combine<'a, C1, S1: Clone, E1, C2, S2: Clone, E2>(
    decider1: Decider<'a, C1, S1, E1>,
    decider2: Decider<'a, C2, S2, E2>,
) -> Decider<'a, Sum<C1, C2>, (S1, S2), Sum<E1, E2>> {
    let new_decide = Box::new(move |c: &Sum<C1, C2>, s: &(S1, S2)| match c {
        Sum::First(c) => {
            let s1 = &s.0;
            let events = (decider1.decide)(c, s1);
            events
                .into_iter()
                .map(|e: E1| Sum::First(e))
                .collect::<Vec<Sum<E1, E2>>>()
        }
        Sum::Second(c) => {
            let s2 = &s.1;
            let events = (decider2.decide)(c, s2);
            events
                .into_iter()
                .map(|e: E2| Sum::Second(e))
                .collect::<Vec<Sum<E1, E2>>>()
        }
    });

    let new_evolve = Box::new(move |s: &(S1, S2), e: &Sum<E1, E2>| match e {
        Sum::First(e) => {
            let s1 = &s.0;
            let new_state = (decider1.evolve)(s1, e);
            (new_state, s.1.to_owned())
        }
        Sum::Second(e) => {
            let s2 = &s.1;
            let new_state = (decider2.evolve)(s2, e);
            (s.0.to_owned(), new_state)
        }
    });

    let new_initial_state = Box::new(move || {
        let s1 = (decider1.initial_state)();
        let s2 = (decider2.initial_state)();
        (s1, s2)
    });

    Decider {
        decide: new_decide,
        evolve: new_evolve,
        initial_state: new_initial_state,
    }
}
