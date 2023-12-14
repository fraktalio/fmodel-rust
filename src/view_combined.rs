use crate::view::View;
use crate::Sum;

/// Combine two views into one.
/// Creates a new instance of a View by combining two views of type `S1`, `E1` and `S2`, `E2` into a new view of type `(S1, S2)`, `Sum<E1, E2>`
pub fn combine<'a, S1: Clone, E1, S2: Clone, E2>(
    view1: View<'a, S1, E1>,
    view2: View<'a, S2, E2>,
) -> View<'a, (S1, S2), Sum<E1, E2>> {
    let new_evolve = Box::new(move |s: &(S1, S2), e: &Sum<E1, E2>| match e {
        Sum::First(e) => {
            let s1 = &s.0;
            let new_state = (view1.evolve)(s1, e);
            (new_state, s.1.to_owned())
        }
        Sum::Second(e) => {
            let s2 = &s.1;
            let new_state = (view2.evolve)(s2, e);
            (s.0.to_owned(), new_state)
        }
    });

    let new_initial_state = Box::new(move || {
        let s1 = (view1.initial_state)();
        let s2 = (view2.initial_state)();
        (s1, s2)
    });

    View {
        evolve: new_evolve,
        initial_state: new_initial_state,
    }
}
