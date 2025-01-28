use crate::{EvolveFunction, InitialStateFunction, Sum, View3, View4, View5, View6};

/// [View] represents the event handling algorithm, responsible for translating the events into denormalized state, which is more adequate for querying.
/// It has two generic parameters `S`/State, `E`/Event , representing the type of the values that View may contain or use.
/// `'a` is used as a lifetime parameter, indicating that all references contained within the struct (e.g., references within the function closures) must have a lifetime that is at least as long as 'a.
///
/// ## Example
/// ```
/// use fmodel_rust::view::View;
///
/// fn view<'a>() -> View<'a, OrderViewState, OrderEvent> {
///     View {
///        // Exhaustive pattern matching is used to handle the events (modeled as Enum - SUM/OR type).
///        evolve: Box::new(|state, event| {
///             let mut new_state = state.clone();
///             match event {
///                OrderEvent::Created(created_event) => {
///                     new_state.order_id = created_event.order_id;
///                     new_state.customer_name = created_event.customer_name.to_owned();
///                     new_state.items = created_event.items.to_owned();
///                 }
///                 OrderEvent::Updated(updated_event) => {
///                     new_state.items = updated_event.updated_items.to_owned();
///                 }
///                 OrderEvent::Cancelled(_) => {
///                     new_state.is_cancelled = true;
///                 }
///             }
///             new_state
///         }),
///         initial_state: Box::new(|| OrderViewState {
///             order_id: 0,
///             customer_name: "".to_string(),
///             items: Vec::new(),
///             is_cancelled: false,
///         }),
///     }
/// }
///
/// #[derive(Debug)]
/// pub enum OrderEvent {
///     Created(OrderCreatedEvent),
///     Updated(OrderUpdatedEvent),
///     Cancelled(OrderCancelledEvent),
/// }
///
/// #[derive(Debug)]
/// pub struct OrderCreatedEvent {
///     pub order_id: u32,
///     pub customer_name: String,
///     pub items: Vec<String>,
/// }
///
/// #[derive(Debug)]
/// pub struct OrderUpdatedEvent {
///     pub order_id: u32,
///     pub updated_items: Vec<String>,
/// }
///
/// #[derive(Debug)]
/// pub struct OrderCancelledEvent {
///     pub order_id: u32,
/// }
///
/// #[derive(Debug, Clone)]
/// struct OrderViewState {
///     order_id: u32,
///     customer_name: String,
///     items: Vec<String>,
///     is_cancelled: bool,
/// }
///
/// let view: View<OrderViewState, OrderEvent> = view();
/// let order_created_event = OrderEvent::Created(OrderCreatedEvent {
///     order_id: 1,
///     customer_name: "John Doe".to_string(),
///     items: vec!["Item 1".to_string(), "Item 2".to_string()],
/// });
/// let new_state = (view.evolve)(&(view.initial_state)(), &order_created_event);
/// ```
pub struct View<'a, S: 'a, E: 'a> {
    /// The `evolve` function is the main state evolution algorithm.
    pub evolve: EvolveFunction<'a, S, E>,
    /// The `initial_state` function is the initial state.
    pub initial_state: InitialStateFunction<'a, S>,
}

impl<'a, S, E> View<'a, S, E> {
    /// Maps the View over the S/State type parameter.
    /// Creates a new instance of [View]`<S2, E>`.
    pub fn map_state<S2, F1, F2>(self, f1: &'a F1, f2: &'a F2) -> View<'a, S2, E>
    where
        F1: Fn(&S2) -> S + Send + Sync,
        F2: Fn(&S) -> S2 + Send + Sync,
    {
        let new_evolve = Box::new(move |s2: &S2, e: &E| {
            let s = f1(s2);
            f2(&(self.evolve)(&s, e))
        });

        let new_initial_state = Box::new(move || f2(&(self.initial_state)()));

        View {
            evolve: new_evolve,
            initial_state: new_initial_state,
        }
    }

    /// Maps the View over the E/Event type parameter.
    /// Creates a new instance of [View]`<S, E2>`.
    pub fn map_event<E2, F>(self, f: &'a F) -> View<'a, S, E2>
    where
        F: Fn(&E2) -> E + Send + Sync,
    {
        let new_evolve = Box::new(move |s: &S, e2: &E2| {
            let e = f(e2);
            (self.evolve)(s, &e)
        });

        let new_initial_state = Box::new(move || (self.initial_state)());

        View {
            evolve: new_evolve,
            initial_state: new_initial_state,
        }
    }

    /// Combines two views into one.
    /// Creates a new instance of a View by combining two views of type `S`, `E` and `S2`, `E2` into a new view of type `(S, S2)`, `Sum<E, E2>`
    /// Combines two views that operate on different event types (`E`` and `E2``) into a new view operating on `Sum<E, E2>`
    #[deprecated(
        since = "0.8.0",
        note = "Use the `merge` function instead. This ensures all your views can subscribe to all `Event`/`E` in the system."
    )]
    pub fn combine<S2, E2>(self, view2: View<'a, S2, E2>) -> View<'a, (S, S2), Sum<E, E2>>
    where
        S: Clone,
        S2: Clone,
    {
        let new_evolve = Box::new(move |s: &(S, S2), e: &Sum<E, E2>| match e {
            Sum::First(e) => {
                let s1 = &s.0;
                let new_state = (self.evolve)(s1, e);
                (new_state, s.1.to_owned())
            }
            Sum::Second(e) => {
                let s2 = &s.1;
                let new_state = (view2.evolve)(s2, e);
                (s.0.to_owned(), new_state)
            }
        });

        let new_initial_state = Box::new(move || {
            let s1 = (self.initial_state)();
            let s2 = (view2.initial_state)();
            (s1, s2)
        });

        View {
            evolve: new_evolve,
            initial_state: new_initial_state,
        }
    }

    /// Merges two views into one.
    /// Creates a new instance of a View by merging two views of type `S`, `E` and `S2`, `E` into a new view of type `(S, S2)`, `E`
    /// Similar to `combine`, but the event type is the same for both views.
    /// This ensures all your views can subscribe to all `Event`/`E` in the system.
    pub fn merge<S2>(self, view2: View<'a, S2, E>) -> View<'a, (S, S2), E>
    where
        S: Clone,
        S2: Clone,
    {
        let new_evolve = Box::new(move |s: &(S, S2), e: &E| {
            let s1 = &s.0;
            let s2 = &s.1;

            let new_state = (self.evolve)(s1, e);
            let new_state2 = (view2.evolve)(s2, e);
            (new_state, new_state2)
        });

        let new_initial_state = Box::new(move || {
            let s1 = (self.initial_state)();
            let s2 = (view2.initial_state)();
            (s1, s2)
        });

        View {
            evolve: new_evolve,
            initial_state: new_initial_state,
        }
    }

    /// Merges three views into one.
    pub fn merge3<S2, S3>(
        self,
        view2: View<'a, S2, E>,
        view3: View<'a, S3, E>,
    ) -> View3<'a, S, S2, S3, E>
    where
        S: Clone,
        S2: Clone,
        S3: Clone,
    {
        self.merge(view2).merge(view3).map_state(
            &|s: &(S, S2, S3)| ((s.0.clone(), s.1.clone()), s.2.clone()),
            &|s: &((S, S2), S3)| (s.0 .0.clone(), s.0 .1.clone(), s.1.clone()),
        )
    }

    /// Merges four views into one.
    pub fn merge4<S2, S3, S4>(
        self,
        view2: View<'a, S2, E>,
        view3: View<'a, S3, E>,
        view4: View<'a, S4, E>,
    ) -> View4<'a, S, S2, S3, S4, E>
    where
        S: Clone,
        S2: Clone,
        S3: Clone,
        S4: Clone,
    {
        self.merge(view2).merge(view3).merge(view4).map_state(
            &|s: &(S, S2, S3, S4)| (((s.0.clone(), s.1.clone()), s.2.clone()), s.3.clone()),
            &|s: &(((S, S2), S3), S4)| {
                (
                    s.0 .0 .0.clone(),
                    s.0 .0 .1.clone(),
                    s.0 .1.clone(),
                    s.1.clone(),
                )
            },
        )
    }

    #[allow(clippy::type_complexity)]
    /// Merges five views into one.
    pub fn merge5<S2, S3, S4, S5>(
        self,
        view2: View<'a, S2, E>,
        view3: View<'a, S3, E>,
        view4: View<'a, S4, E>,
        view5: View<'a, S5, E>,
    ) -> View5<'a, S, S2, S3, S4, S5, E>
    where
        S: Clone,
        S2: Clone,
        S3: Clone,
        S4: Clone,
        S5: Clone,
    {
        self.merge(view2)
            .merge(view3)
            .merge(view4)
            .merge(view5)
            .map_state(
                &|s: &(S, S2, S3, S4, S5)| {
                    (
                        (((s.0.clone(), s.1.clone()), s.2.clone()), s.3.clone()),
                        s.4.clone(),
                    )
                },
                &|s: &((((S, S2), S3), S4), S5)| {
                    (
                        s.0 .0 .0 .0.clone(),
                        s.0 .0 .0 .1.clone(),
                        s.0 .0 .1.clone(),
                        s.0 .1.clone(),
                        s.1.clone(),
                    )
                },
            )
    }

    #[allow(clippy::type_complexity)]
    /// Merges six views into one.
    pub fn merge6<S2, S3, S4, S5, S6>(
        self,
        view2: View<'a, S2, E>,
        view3: View<'a, S3, E>,
        view4: View<'a, S4, E>,
        view5: View<'a, S5, E>,
        view6: View<'a, S6, E>,
    ) -> View6<'a, S, S2, S3, S4, S5, S6, E>
    where
        S: Clone,
        S2: Clone,
        S3: Clone,
        S4: Clone,
        S5: Clone,
        S6: Clone,
    {
        self.merge(view2)
            .merge(view3)
            .merge(view4)
            .merge(view5)
            .merge(view6)
            .map_state(
                &|s: &(S, S2, S3, S4, S5, S6)| {
                    (
                        (
                            (((s.0.clone(), s.1.clone()), s.2.clone()), s.3.clone()),
                            s.4.clone(),
                        ),
                        s.5.clone(),
                    )
                },
                &|s: &(((((S, S2), S3), S4), S5), S6)| {
                    (
                        s.0 .0 .0 .0 .0.clone(),
                        s.0 .0 .0 .0 .1.clone(),
                        s.0 .0 .0 .1.clone(),
                        s.0 .0 .1.clone(),
                        s.0 .1.clone(),
                        s.1.clone(),
                    )
                },
            )
    }
}

/// Formalizes the `State Computation` algorithm for the `view` to handle events based on the current state, and produce new state.
pub trait ViewStateComputation<E, S> {
    /// Computes new state based on the current state and the events.
    fn compute_new_state(&self, current_state: Option<S>, events: &[&E]) -> S;
}

impl<S, E> ViewStateComputation<E, S> for View<'_, S, E> {
    /// Computes new state based on the current state and the events.
    fn compute_new_state(&self, current_state: Option<S>, events: &[&E]) -> S {
        let effective_current_state = current_state.unwrap_or_else(|| (self.initial_state)());
        events.iter().fold(effective_current_state, |state, event| {
            (self.evolve)(&state, event)
        })
    }
}
