#[cfg(feature = "not-send-futures")]
use std::rc::Rc;
#[cfg(not(feature = "not-send-futures"))]
use std::sync::Arc;

use crate::{
    DecideFunction, Decider3, Decider4, Decider5, Decider6, EvolveFunction, InitialStateFunction,
    Sum, Sum3, Sum4, Sum5, Sum6,
};

/// [Decider] represents the main decision-making algorithm.
/// It has three generic parameters `C`/`Command`, `S`/`State`, `E`/`Event` , representing the type of the values that Decider may contain or use.
/// `'a` is used as a lifetime parameter, indicating that all references contained within the struct (e.g., references within the function closures) must have a lifetime that is at least as long as 'a.
///
/// ## Example
/// ```
/// use fmodel_rust::decider::{Decider, EventComputation, StateComputation};
///
/// fn decider<'a>() -> Decider<'a, OrderCommand, OrderState, OrderEvent> {
///     Decider {
///         // Exhaustive pattern matching is used to handle the commands (modeled as Enum - SUM/OR type).
///         decide: Box::new(|command, state| {
///            match command {
///                 OrderCommand::Create(create_cmd) => {
///                     Ok(vec![OrderEvent::Created(OrderCreatedEvent {
///                         order_id: create_cmd.order_id,
///                         customer_name: create_cmd.customer_name.to_owned(),
///                         items: create_cmd.items.to_owned(),
///                     })])
///                 }
///                 OrderCommand::Update(update_cmd) => {
///                     if state.order_id == update_cmd.order_id {
///                         Ok(vec![OrderEvent::Updated(OrderUpdatedEvent {
///                             order_id: update_cmd.order_id,
///                             updated_items: update_cmd.new_items.to_owned(),
///                         })])
///                     } else {
///                         Ok(vec![])
///                     }
///                 }
///                 OrderCommand::Cancel(cancel_cmd) => {
///                     if state.order_id == cancel_cmd.order_id {
///                         Ok(vec![OrderEvent::Cancelled(OrderCancelledEvent {
///                             order_id: cancel_cmd.order_id,
///                         })])
///                     } else {
///                         Ok(vec![])
///                     }
///                 }
///             }
///         }),
///         // Exhaustive pattern matching is used to handle the events (modeled as Enum - SUM/OR type).
///         evolve: Box::new(|state, event| {
///             let mut new_state = state.clone();
///             match event {
///                 OrderEvent::Created(created_event) => {
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
///         initial_state: Box::new(|| OrderState {
///             order_id: 0,
///             customer_name: "".to_string(),
///             items: Vec::new(),
///             is_cancelled: false,
///         }),
///     }
/// }
///
/// // Modeling the commands, events, and state. Enum is modeling the SUM/OR type, and struct is modeling the PRODUCT/AND type.
/// #[derive(Debug)]
/// pub enum OrderCommand {
///     Create(CreateOrderCommand),
///     Update(UpdateOrderCommand),
///     Cancel(CancelOrderCommand),
/// }
///
/// #[derive(Debug)]
/// pub struct CreateOrderCommand {
///     pub order_id: u32,
///     pub customer_name: String,
///     pub items: Vec<String>,
/// }
///
/// #[derive(Debug)]
/// pub struct UpdateOrderCommand {
///     pub order_id: u32,
///     pub new_items: Vec<String>,
/// }
///
/// #[derive(Debug)]
/// pub struct CancelOrderCommand {
///     pub order_id: u32,
/// }
///
/// #[derive(Debug, PartialEq)]
/// pub enum OrderEvent {
///     Created(OrderCreatedEvent),
///     Updated(OrderUpdatedEvent),
///     Cancelled(OrderCancelledEvent),
/// }
///
/// #[derive(Debug, PartialEq)]
/// pub struct OrderCreatedEvent {
///     pub order_id: u32,
///     pub customer_name: String,
///     pub items: Vec<String>,
/// }
///
/// #[derive(Debug, PartialEq)]
/// pub struct OrderUpdatedEvent {
///     pub order_id: u32,
///     pub updated_items: Vec<String>,
/// }
///
/// #[derive(Debug, PartialEq)]
/// pub struct OrderCancelledEvent {
///     pub order_id: u32,
/// }
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct OrderState {
///     order_id: u32,
///     customer_name: String,
///     items: Vec<String>,
///     is_cancelled: bool,
/// }
///
/// let decider: Decider<OrderCommand, OrderState, OrderEvent> = decider();
/// let create_order_command = OrderCommand::Create(CreateOrderCommand {
///     order_id: 1,
///     customer_name: "John Doe".to_string(),
///     items: vec!["Item 1".to_string(), "Item 2".to_string()],
/// });
/// let new_events = decider.compute_new_events(&[], &create_order_command);
///     assert_eq!(new_events, Ok(vec![OrderEvent::Created(OrderCreatedEvent {
///         order_id: 1,
///         customer_name: "John Doe".to_string(),
///         items: vec!["Item 1".to_string(), "Item 2".to_string()],
///     })]));
///     let new_state = decider.compute_new_state(None, &create_order_command);
///     assert_eq!(new_state, Ok(OrderState {
///         order_id: 1,
///         customer_name: "John Doe".to_string(),
///         items: vec!["Item 1".to_string(), "Item 2".to_string()],
///         is_cancelled: false,
///     }));
///
/// ```
pub struct Decider<'a, C: 'a, S: 'a, E: 'a, Error: 'a = ()> {
    /// The `decide` function is used to decide which events to produce based on the command and the current state.
    pub decide: DecideFunction<'a, C, S, E, Error>,
    /// The `evolve` function is used to evolve the state based on the current state and the event.
    pub evolve: EvolveFunction<'a, S, E>,
    /// The `initial_state` function is used to produce the initial state of the decider.
    pub initial_state: InitialStateFunction<'a, S>,
}

impl<'a, C, S, E, Error> Decider<'a, C, S, E, Error> {
    /// Maps the Decider over the S/State type parameter.
    /// Creates a new instance of [Decider]`<C, S2, E, Error>`.
    #[cfg(not(feature = "not-send-futures"))]
    pub fn map_state<S2, F1, F2>(self, f1: F1, f2: F2) -> Decider<'a, C, S2, E, Error>
    where
        F1: Fn(&S2) -> S + Send + Sync + 'a,
        F2: Fn(&S) -> S2 + Send + Sync + 'a,
    {
        let f1 = Arc::new(f1);
        let f2 = Arc::new(f2);

        let new_decide = {
            let f1 = Arc::clone(&f1);
            Box::new(move |c: &C, s2: &S2| {
                let s = f1(s2);
                (self.decide)(c, &s)
            })
        };

        let new_evolve = {
            let f2 = Arc::clone(&f2);
            Box::new(move |s2: &S2, e: &E| {
                let s = f1(s2);
                f2(&(self.evolve)(&s, e))
            })
        };

        let new_initial_state = { Box::new(move || f2(&(self.initial_state)())) };

        Decider {
            decide: new_decide,
            evolve: new_evolve,
            initial_state: new_initial_state,
        }
    }

    /// Maps the Decider over the S/State type parameter.
    /// Creates a new instance of [Decider]`<C, S2, E, Error>`.
    #[cfg(feature = "not-send-futures")]
    pub fn map_state<S2, F1, F2>(self, f1: F1, f2: F2) -> Decider<'a, C, S2, E, Error>
    where
        F1: Fn(&S2) -> S + 'a,
        F2: Fn(&S) -> S2 + 'a,
    {
        let f1 = Rc::new(f1);
        let f2 = Rc::new(f2);

        let new_decide = {
            let f1 = Rc::clone(&f1);
            Box::new(move |c: &C, s2: &S2| {
                let s = f1(s2);
                (self.decide)(c, &s)
            })
        };

        let new_evolve = {
            let f2 = Rc::clone(&f2);
            Box::new(move |s2: &S2, e: &E| {
                let s = f1(s2);
                f2(&(self.evolve)(&s, e))
            })
        };

        let new_initial_state = { Box::new(move || f2(&(self.initial_state)())) };

        Decider {
            decide: new_decide,
            evolve: new_evolve,
            initial_state: new_initial_state,
        }
    }
    /// Maps the Decider over the E/Event type parameter.
    /// Creates a new instance of [Decider]`<C, S, E2, Error>`.
    #[cfg(not(feature = "not-send-futures"))]
    pub fn map_event<E2, F1, F2>(self, f1: F1, f2: F2) -> Decider<'a, C, S, E2, Error>
    where
        F1: Fn(&E2) -> E + Send + Sync + 'a,
        F2: Fn(&E) -> E2 + Send + Sync + 'a,
    {
        let new_decide = Box::new(move |c: &C, s: &S| {
            (self.decide)(c, s).map(|result| result.into_iter().map(|e: E| f2(&e)).collect())
        });

        let new_evolve = Box::new(move |s: &S, e2: &E2| {
            let e = f1(e2);
            (self.evolve)(s, &e)
        });

        let new_initial_state = Box::new(move || (self.initial_state)());

        Decider {
            decide: new_decide,
            evolve: new_evolve,
            initial_state: new_initial_state,
        }
    }

    /// Maps the Decider over the E/Event type parameter.
    /// Creates a new instance of [Decider]`<C, S, E2, Error>`.
    #[cfg(feature = "not-send-futures")]
    pub fn map_event<E2, F1, F2>(self, f1: F1, f2: F2) -> Decider<'a, C, S, E2, Error>
    where
        F1: Fn(&E2) -> E + 'a,
        F2: Fn(&E) -> E2 + 'a,
    {
        let new_decide = Box::new(move |c: &C, s: &S| {
            (self.decide)(c, s).map(|result| result.into_iter().map(|e: E| f2(&e)).collect())
        });

        let new_evolve = Box::new(move |s: &S, e2: &E2| {
            let e = f1(e2);
            (self.evolve)(s, &e)
        });

        let new_initial_state = Box::new(move || (self.initial_state)());

        Decider {
            decide: new_decide,
            evolve: new_evolve,
            initial_state: new_initial_state,
        }
    }

    /// Maps the Decider over the C/Command type parameter.
    /// Creates a new instance of [Decider]`<C2, S, E, Error>`.
    #[cfg(not(feature = "not-send-futures"))]
    pub fn map_command<C2, F>(self, f: F) -> Decider<'a, C2, S, E, Error>
    where
        F: Fn(&C2) -> C + Send + Sync + 'a,
    {
        let new_decide = Box::new(move |c2: &C2, s: &S| {
            let c = f(c2);
            (self.decide)(&c, s)
        });

        let new_evolve = Box::new(move |s: &S, e: &E| (self.evolve)(s, e));

        let new_initial_state = Box::new(move || (self.initial_state)());

        Decider {
            decide: new_decide,
            evolve: new_evolve,
            initial_state: new_initial_state,
        }
    }

    /// Maps the Decider over the C/Command type parameter.
    /// Creates a new instance of [Decider]`<C2, S, E, Error>`.
    #[cfg(feature = "not-send-futures")]
    pub fn map_command<C2, F>(self, f: F) -> Decider<'a, C2, S, E, Error>
    where
        F: Fn(&C2) -> C + 'a,
    {
        let new_decide = Box::new(move |c2: &C2, s: &S| {
            let c = f(c2);
            (self.decide)(&c, s)
        });

        let new_evolve = Box::new(move |s: &S, e: &E| (self.evolve)(s, e));

        let new_initial_state = Box::new(move || (self.initial_state)());

        Decider {
            decide: new_decide,
            evolve: new_evolve,
            initial_state: new_initial_state,
        }
    }

    /// Maps the Decider over the Error type parameter.
    /// Creates a new instance of [Decider]`<C, S, E, Error2>`.
    #[cfg(not(feature = "not-send-futures"))]
    pub fn map_error<Error2, F>(self, f: F) -> Decider<'a, C, S, E, Error2>
    where
        F: Fn(&Error) -> Error2 + Send + Sync + 'a,
    {
        let new_decide = Box::new(move |c: &C, s: &S| (self.decide)(c, s).map_err(|e| f(&e)));

        let new_evolve = Box::new(move |s: &S, e: &E| (self.evolve)(s, e));

        let new_initial_state = Box::new(move || (self.initial_state)());

        Decider {
            decide: new_decide,
            evolve: new_evolve,
            initial_state: new_initial_state,
        }
    }

    /// Maps the Decider over the Error type parameter.
    /// Creates a new instance of [Decider]`<C, S, E, Error2>`.
    #[cfg(feature = "not-send-futures")]
    pub fn map_error<Error2, F>(self, f: F) -> Decider<'a, C, S, E, Error2>
    where
        F: Fn(&Error) -> Error2 + 'a,
    {
        let new_decide = Box::new(move |c: &C, s: &S| (self.decide)(c, s).map_err(|e| f(&e)));

        let new_evolve = Box::new(move |s: &S, e: &E| (self.evolve)(s, e));

        let new_initial_state = Box::new(move || (self.initial_state)());

        Decider {
            decide: new_decide,
            evolve: new_evolve,
            initial_state: new_initial_state,
        }
    }

    /// Combines two deciders into one bigger decider
    /// Creates a new instance of a Decider by combining two deciders of type `C`, `S`, `E` and `C2`, `S2`, `E2` into a new decider of type `Sum<C, C2>`, `(S, S2)`, `Sum<E, E2>`
    #[allow(clippy::type_complexity)]
    pub fn combine<C2, S2, E2>(
        self,
        decider2: Decider<'a, C2, S2, E2, Error>,
    ) -> Decider<'a, Sum<C, C2>, (S, S2), Sum<E, E2>, Error>
    where
        S: Clone,
        S2: Clone,
    {
        let new_decide = Box::new(move |c: &Sum<C, C2>, s: &(S, S2)| match c {
            Sum::First(c) => {
                let s1 = &s.0;
                let events = (self.decide)(c, s1);
                events.map(|result| {
                    result
                        .into_iter()
                        .map(|e: E| Sum::First(e))
                        .collect::<Vec<Sum<E, E2>>>()
                })
            }
            Sum::Second(c) => {
                let s2 = &s.1;
                let events = (decider2.decide)(c, s2);
                events.map(|result| {
                    result
                        .into_iter()
                        .map(|e: E2| Sum::Second(e))
                        .collect::<Vec<Sum<E, E2>>>()
                })
            }
        });

        let new_evolve = Box::new(move |s: &(S, S2), e: &Sum<E, E2>| match e {
            Sum::First(e) => {
                let s1 = &s.0;
                let new_state = (self.evolve)(s1, e);
                (new_state, s.1.to_owned())
            }
            Sum::Second(e) => {
                let s2 = &s.1;
                let new_state = (decider2.evolve)(s2, e);
                (s.0.to_owned(), new_state)
            }
        });

        let new_initial_state = Box::new(move || {
            let s1 = (self.initial_state)();
            let s2 = (decider2.initial_state)();
            (s1, s2)
        });

        Decider {
            decide: new_decide,
            evolve: new_evolve,
            initial_state: new_initial_state,
        }
    }

    /// Combines three deciders into one bigger decider
    pub fn combine3<C2, S2, E2, C3, S3, E3>(
        self,
        decider2: Decider<'a, C2, S2, E2, Error>,
        decider3: Decider<'a, C3, S3, E3, Error>,
    ) -> Decider3<'a, C, C2, C3, S, S2, S3, E, E2, E3, Error>
    where
        S: Clone,
        S2: Clone,
        S3: Clone,
        E: Clone,
        E2: Clone,
        E3: Clone,
        C: Clone,
        C2: Clone,
        C3: Clone,
    {
        // First combine self with decider2
        let combined = self.combine(decider2);

        // Then combine with decider3 and map the types
        combined
            .combine(decider3)
            .map_state(
                |s: &(S, S2, S3)| ((s.0.clone(), s.1.clone()), s.2.clone()),
                |s: &((S, S2), S3)| (s.0 .0.clone(), s.0 .1.clone(), s.1.clone()),
            )
            .map_event(
                |e: &Sum3<E, E2, E3>| match e {
                    Sum3::First(ref e) => Sum::First(Sum::First(e.clone())),
                    Sum3::Second(ref e) => Sum::First(Sum::Second(e.clone())),
                    Sum3::Third(ref e) => Sum::Second(e.clone()),
                },
                |e: &Sum<Sum<E, E2>, E3>| match e {
                    Sum::First(Sum::First(e)) => Sum3::First(e.clone()),
                    Sum::First(Sum::Second(e)) => Sum3::Second(e.clone()),
                    Sum::Second(e) => Sum3::Third(e.clone()),
                },
            )
            .map_command(|c: &Sum3<C, C2, C3>| match c {
                Sum3::First(c) => Sum::First(Sum::First(c.clone())),
                Sum3::Second(c) => Sum::First(Sum::Second(c.clone())),
                Sum3::Third(c) => Sum::Second(c.clone()),
            })
    }

    #[allow(clippy::type_complexity)]
    /// Combines four deciders into one bigger decider
    pub fn combine4<C2, S2, E2, C3, S3, E3, C4, S4, E4>(
        self,
        decider2: Decider<'a, C2, S2, E2, Error>,
        decider3: Decider<'a, C3, S3, E3, Error>,
        decider4: Decider<'a, C4, S4, E4, Error>,
    ) -> Decider4<'a, C, C2, C3, C4, S, S2, S3, S4, E, E2, E3, E4, Error>
    where
        S: Clone,
        S2: Clone,
        S3: Clone,
        S4: Clone,
        E: Clone,
        E2: Clone,
        E3: Clone,
        E4: Clone,
        C: Clone,
        C2: Clone,
        C3: Clone,
        C4: Clone,
    {
        let combined = self
            .combine(decider2)
            .combine(decider3)
            .combine(decider4)
            .map_state(
                |s: &(S, S2, S3, S4)| (((s.0.clone(), s.1.clone()), s.2.clone()), s.3.clone()),
                |s: &(((S, S2), S3), S4)| {
                    (
                        s.0 .0 .0.clone(),
                        s.0 .0 .1.clone(),
                        s.0 .1.clone(),
                        s.1.clone(),
                    )
                },
            )
            .map_event(
                |e: &Sum4<E, E2, E3, E4>| match e {
                    Sum4::First(e) => Sum::First(Sum::First(Sum::First(e.clone()))),
                    Sum4::Second(e) => Sum::First(Sum::First(Sum::Second(e.clone()))),
                    Sum4::Third(e) => Sum::First(Sum::Second(e.clone())),
                    Sum4::Fourth(e) => Sum::Second(e.clone()),
                },
                |e: &Sum<Sum<Sum<E, E2>, E3>, E4>| match e {
                    Sum::First(Sum::First(Sum::First(e))) => Sum4::First(e.clone()),
                    Sum::First(Sum::First(Sum::Second(e))) => Sum4::Second(e.clone()),
                    Sum::First(Sum::Second(e)) => Sum4::Third(e.clone()),
                    Sum::Second(e) => Sum4::Fourth(e.clone()),
                },
            )
            .map_command(|c: &Sum4<C, C2, C3, C4>| match c {
                Sum4::First(c) => Sum::First(Sum::First(Sum::First(c.clone()))),
                Sum4::Second(c) => Sum::First(Sum::First(Sum::Second(c.clone()))),
                Sum4::Third(c) => Sum::First(Sum::Second(c.clone())),
                Sum4::Fourth(c) => Sum::Second(c.clone()),
            });
        combined
    }

    #[allow(clippy::type_complexity)]
    /// Combines five deciders into one bigger decider
    pub fn combine5<C2, S2, E2, C3, S3, E3, C4, S4, E4, C5, S5, E5>(
        self,
        decider2: Decider<'a, C2, S2, E2, Error>,
        decider3: Decider<'a, C3, S3, E3, Error>,
        decider4: Decider<'a, C4, S4, E4, Error>,
        decider5: Decider<'a, C5, S5, E5, Error>,
    ) -> Decider5<'a, C, C2, C3, C4, C5, S, S2, S3, S4, S5, E, E2, E3, E4, E5, Error>
    where
        S: Clone,
        S2: Clone,
        S3: Clone,
        S4: Clone,
        S5: Clone,
        E: Clone,
        E2: Clone,
        E3: Clone,
        E4: Clone,
        E5: Clone,
        C: Clone,
        C2: Clone,
        C3: Clone,
        C4: Clone,
        C5: Clone,
    {
        let combined = self
            .combine(decider2)
            .combine(decider3)
            .combine(decider4)
            .combine(decider5)
            .map_state(
                |s: &(S, S2, S3, S4, S5)| {
                    (
                        (((s.0.clone(), s.1.clone()), s.2.clone()), s.3.clone()),
                        s.4.clone(),
                    )
                },
                |s: &((((S, S2), S3), S4), S5)| {
                    (
                        s.0 .0 .0 .0.clone(),
                        s.0 .0 .0 .1.clone(),
                        s.0 .0 .1.clone(),
                        s.0 .1.clone(),
                        s.1.clone(),
                    )
                },
            )
            .map_event(
                |e: &Sum5<E, E2, E3, E4, E5>| match e {
                    Sum5::First(e) => Sum::First(Sum::First(Sum::First(Sum::First(e.clone())))),
                    Sum5::Second(e) => Sum::First(Sum::First(Sum::First(Sum::Second(e.clone())))),
                    Sum5::Third(e) => Sum::First(Sum::First(Sum::Second(e.clone()))),
                    Sum5::Fourth(e) => Sum::First(Sum::Second(e.clone())),
                    Sum5::Fifth(e) => Sum::Second(e.clone()),
                },
                |e: &Sum<Sum<Sum<Sum<E, E2>, E3>, E4>, E5>| match e {
                    Sum::First(Sum::First(Sum::First(Sum::First(e)))) => Sum5::First(e.clone()),
                    Sum::First(Sum::First(Sum::First(Sum::Second(e)))) => Sum5::Second(e.clone()),
                    Sum::First(Sum::First(Sum::Second(e))) => Sum5::Third(e.clone()),
                    Sum::First(Sum::Second(e)) => Sum5::Fourth(e.clone()),
                    Sum::Second(e) => Sum5::Fifth(e.clone()),
                },
            )
            .map_command(|c: &Sum5<C, C2, C3, C4, C5>| match c {
                Sum5::First(c) => Sum::First(Sum::First(Sum::First(Sum::First(c.clone())))),
                Sum5::Second(c) => Sum::First(Sum::First(Sum::First(Sum::Second(c.clone())))),
                Sum5::Third(c) => Sum::First(Sum::First(Sum::Second(c.clone()))),
                Sum5::Fourth(c) => Sum::First(Sum::Second(c.clone())),
                Sum5::Fifth(c) => Sum::Second(c.clone()),
            });
        combined
    }

    #[allow(clippy::type_complexity)]
    /// Combines six deciders into one bigger decider
    pub fn combine6<C2, S2, E2, C3, S3, E3, C4, S4, E4, C5, S5, E5, C6, S6, E6>(
        self,
        decider2: Decider<'a, C2, S2, E2, Error>,
        decider3: Decider<'a, C3, S3, E3, Error>,
        decider4: Decider<'a, C4, S4, E4, Error>,
        decider5: Decider<'a, C5, S5, E5, Error>,
        decider6: Decider<'a, C6, S6, E6, Error>,
    ) -> Decider6<'a, C, C2, C3, C4, C5, C6, S, S2, S3, S4, S5, S6, E, E2, E3, E4, E5, E6, Error>
    where
        S: Clone,
        S2: Clone,
        S3: Clone,
        S4: Clone,
        S5: Clone,
        S6: Clone,
        E: Clone,
        E2: Clone,
        E3: Clone,
        E4: Clone,
        E5: Clone,
        E6: Clone,
        C: Clone,
        C2: Clone,
        C3: Clone,
        C4: Clone,
        C5: Clone,
        C6: Clone,
    {
        let combined = self
            .combine(decider2)
            .combine(decider3)
            .combine(decider4)
            .combine(decider5)
            .combine(decider6)
            .map_state(
                |s: &(S, S2, S3, S4, S5, S6)| {
                    (
                        (
                            (((s.0.clone(), s.1.clone()), s.2.clone()), s.3.clone()),
                            s.4.clone(),
                        ),
                        s.5.clone(),
                    )
                },
                |s: &(((((S, S2), S3), S4), S5), S6)| {
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
            .map_event(
                |e: &Sum6<E, E2, E3, E4, E5, E6>| match e {
                    Sum6::First(e) => {
                        Sum::First(Sum::First(Sum::First(Sum::First(Sum::First(e.clone())))))
                    }
                    Sum6::Second(e) => {
                        Sum::First(Sum::First(Sum::First(Sum::First(Sum::Second(e.clone())))))
                    }
                    Sum6::Third(e) => Sum::First(Sum::First(Sum::First(Sum::Second(e.clone())))),
                    Sum6::Fourth(e) => Sum::First(Sum::First(Sum::Second(e.clone()))),
                    Sum6::Fifth(e) => Sum::First(Sum::Second(e.clone())),
                    Sum6::Sixth(e) => Sum::Second(e.clone()),
                },
                |e: &Sum<Sum<Sum<Sum<Sum<E, E2>, E3>, E4>, E5>, E6>| match e {
                    Sum::First(Sum::First(Sum::First(Sum::First(Sum::First(e))))) => {
                        Sum6::First(e.clone())
                    }
                    Sum::First(Sum::First(Sum::First(Sum::First(Sum::Second(e))))) => {
                        Sum6::Second(e.clone())
                    }
                    Sum::First(Sum::First(Sum::First(Sum::Second(e)))) => Sum6::Third(e.clone()),
                    Sum::First(Sum::First(Sum::Second(e))) => Sum6::Fourth(e.clone()),
                    Sum::First(Sum::Second(e)) => Sum6::Fifth(e.clone()),
                    Sum::Second(e) => Sum6::Sixth(e.clone()),
                },
            )
            .map_command(|c: &Sum6<C, C2, C3, C4, C5, C6>| match c {
                Sum6::First(c) => {
                    Sum::First(Sum::First(Sum::First(Sum::First(Sum::First(c.clone())))))
                }
                Sum6::Second(c) => {
                    Sum::First(Sum::First(Sum::First(Sum::First(Sum::Second(c.clone())))))
                }
                Sum6::Third(c) => Sum::First(Sum::First(Sum::First(Sum::Second(c.clone())))),
                Sum6::Fourth(c) => Sum::First(Sum::First(Sum::Second(c.clone()))),
                Sum6::Fifth(c) => Sum::First(Sum::Second(c.clone())),
                Sum6::Sixth(c) => Sum::Second(c.clone()),
            });
        combined
    }
}

/// Formalizes the `Event Computation` algorithm / event sourced system for the `decider` to handle commands based on the current events, and produce new events.
pub trait EventComputation<C, S, E, Error = ()> {
    /// Computes new events based on the current events and the command.
    fn compute_new_events(&self, current_events: &[E], command: &C) -> Result<Vec<E>, Error>;
}

/// Formalizes the `State Computation` algorithm / state-stored system for the `decider` to handle commands based on the current state, and produce new state.
pub trait StateComputation<C, S, E, Error = ()> {
    /// Computes new state based on the current state and the command.
    fn compute_new_state(&self, current_state: Option<S>, command: &C) -> Result<S, Error>;
}

impl<C, S, E, Error> EventComputation<C, S, E, Error> for Decider<'_, C, S, E, Error> {
    /// Computes new events based on the current events and the command.
    fn compute_new_events(&self, current_events: &[E], command: &C) -> Result<Vec<E>, Error> {
        let current_state: S = current_events
            .iter()
            .fold((self.initial_state)(), |state, event| {
                (self.evolve)(&state, event)
            });
        (self.decide)(command, &current_state)
    }
}

impl<C, S, E, Error> StateComputation<C, S, E, Error> for Decider<'_, C, S, E, Error> {
    /// Computes new state based on the current state and the command.
    fn compute_new_state(&self, current_state: Option<S>, command: &C) -> Result<S, Error> {
        let effective_current_state = current_state.unwrap_or_else(|| (self.initial_state)());
        let events = (self.decide)(command, &effective_current_state);
        events.map(|result| {
            result
                .into_iter()
                .fold(effective_current_state, |state, event| {
                    (self.evolve)(&state, &event)
                })
        })
    }
}
