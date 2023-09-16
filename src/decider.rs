use crate::{DecideFunction, EvolveFunction, InitialStateFunction};

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
///                     vec![OrderEvent::Created(OrderCreatedEvent {
///                         order_id: create_cmd.order_id,
///                         customer_name: create_cmd.customer_name.to_owned(),
///                         items: create_cmd.items.to_owned(),
///                     })]
///                 }
///                 OrderCommand::Update(update_cmd) => {
///                     if state.order_id == update_cmd.order_id {
///                         vec![OrderEvent::Updated(OrderUpdatedEvent {
///                             order_id: update_cmd.order_id,
///                             updated_items: update_cmd.new_items.to_owned(),
///                         })]
///                     } else {
///                         vec![]
///                     }
///                 }
///                 OrderCommand::Cancel(cancel_cmd) => {
///                     if state.order_id == cancel_cmd.order_id {
///                         vec![OrderEvent::Cancelled(OrderCancelledEvent {
///                             order_id: cancel_cmd.order_id,
///                         })]
///                     } else {
///                         vec![]
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
///     assert_eq!(new_events, [OrderEvent::Created(OrderCreatedEvent {
///         order_id: 1,
///         customer_name: "John Doe".to_string(),
///         items: vec!["Item 1".to_string(), "Item 2".to_string()],
///     })]);
///     let new_state = decider.compute_new_state(None, &create_order_command);
///     assert_eq!(new_state, OrderState {
///         order_id: 1,
///         customer_name: "John Doe".to_string(),
///         items: vec!["Item 1".to_string(), "Item 2".to_string()],
///         is_cancelled: false,
///     });
///
/// ```
pub struct Decider<'a, C: 'a, S: 'a, E: 'a> {
    /// The `decide` function is used to decide which events to produce based on the command and the current state.
    pub decide: DecideFunction<'a, C, S, E>,
    /// The `evolve` function is used to evolve the state based on the current state and the event.
    pub evolve: EvolveFunction<'a, S, E>,
    /// The `initial_state` function is used to produce the initial state of the decider.
    pub initial_state: InitialStateFunction<'a, S>,
}

impl<'a, C, S, E> Decider<'a, C, S, E> {
    /// Maps the Decider over the S/State type parameter.
    /// Creates a new instance of [Decider]`<C, S2, E>`.
    pub fn map_state<S2, F1, F2>(self, f1: &'a F1, f2: &'a F2) -> Decider<'a, C, S2, E>
    where
        F1: Fn(&S2) -> S + Send + Sync,
        F2: Fn(&S) -> S2 + Send + Sync,
    {
        let new_decide = Box::new(move |c: &C, s2: &S2| {
            let s = f1(s2);
            (self.decide)(c, &s)
        });

        let new_evolve = Box::new(move |s2: &S2, e: &E| {
            let s = f1(s2);
            f2(&(self.evolve)(&s, e))
        });

        let new_initial_state = Box::new(move || f2(&(self.initial_state)()));

        Decider {
            decide: new_decide,
            evolve: new_evolve,
            initial_state: new_initial_state,
        }
    }

    /// Maps the Decider over the E/Event type parameter.
    /// Creates a new instance of [Decider]`<C, S, E2>`.
    pub fn map_event<E2, F1, F2>(self, f1: &'a F1, f2: &'a F2) -> Decider<'a, C, S, E2>
    where
        F1: Fn(&E2) -> E + Send + Sync,
        F2: Fn(&E) -> E2 + Send + Sync,
    {
        let new_decide = Box::new(move |c: &C, s: &S| {
            (self.decide)(c, s).into_iter().map(|e: E| f2(&e)).collect()
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
    /// Creates a new instance of [Decider]`<C2, S, E>`.
    pub fn map_command<C2, F>(self, f: &'a F) -> Decider<'a, C2, S, E>
    where
        F: Fn(&C2) -> C + Send + Sync,
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
}

/// Formalizes the `Event Computation` algorithm / event sourced system for the `decider` to handle commands based on the current events, and produce new events.
pub trait EventComputation<C, E> {
    /// Computes new events based on the current events and the command.
    fn compute_new_events(&self, current_events: &[E], command: &C) -> Vec<E>;
}

/// Formalizes the `State Computation` algorithm / state-stored system for the `decider` to handle commands based on the current state, and produce new state.
pub trait StateComputation<C, S> {
    /// Computes new state based on the current state and the command.
    fn compute_new_state(&self, current_state: Option<S>, command: &C) -> S;
}

impl<'a, C, S, E> EventComputation<C, E> for Decider<'a, C, S, E> {
    /// Computes new events based on the current events and the command.
    fn compute_new_events(&self, current_events: &[E], command: &C) -> Vec<E> {
        let current_state: S = current_events
            .iter()
            .fold((self.initial_state)(), |state, event| {
                (self.evolve)(&state, event)
            });
        (self.decide)(command, &current_state)
    }
}

impl<'a, C, S, E> StateComputation<C, S> for Decider<'a, C, S, E> {
    /// Computes new state based on the current state and the command.
    fn compute_new_state(&self, current_state: Option<S>, command: &C) -> S {
        let effective_current_state = current_state.unwrap_or_else(|| (self.initial_state)());
        let events = (self.decide)(command, &effective_current_state);
        events
            .into_iter()
            .fold(effective_current_state, |state, event| {
                (self.evolve)(&state, &event)
            })
    }
}
