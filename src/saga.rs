use crate::ReactFunction;

/// [Saga] is a datatype that represents the central point of control, deciding what to execute next ([A]), based on the action result ([AR]).
/// It has two generic parameters `AR`/Action Result, `A`/Action , representing the type of the values that Saga may contain or use.
/// `'a` is used as a lifetime parameter, indicating that all references contained within the struct (e.g., references within the function closures) must have a lifetime that is at least as long as 'a.
///
/// It is common to consider Event as Action Result, and Command as Action, but it is not mandatory.
/// For example, Action Result can be a request response from a remote service.
///
/// ## Example
///
/// ```
/// use fmodel_rust::saga::Saga;
///
/// fn saga<'a>() -> Saga<'a, OrderEvent, ShipmentCommand> {
///     Saga {
///         react: Box::new(|event| match event {
///             OrderEvent::Created(created_event) => {
///                 vec![ShipmentCommand::Create(CreateShipmentCommand {
///                     shipment_id: created_event.order_id,
///                     order_id: created_event.order_id,
///                     customer_name: created_event.customer_name.to_owned(),
///                     items: created_event.items.to_owned(),
///                 })]
///             }
///             OrderEvent::Updated(_updated_event) => {
///                 vec![]
///             }
///             OrderEvent::Cancelled(_cancelled_event) => {
///                 vec![]
///             }
///         }),
///     }
/// }
///
/// #[derive(Debug, PartialEq)]
/// #[allow(dead_code)]
/// pub enum ShipmentCommand {
///     Create(CreateShipmentCommand),
/// }
///
/// #[derive(Debug, PartialEq)]
/// pub struct CreateShipmentCommand {
///     pub shipment_id: u32,
///     pub order_id: u32,
///     pub customer_name: String,
///     pub items: Vec<String>,
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
/// let saga: Saga<OrderEvent, ShipmentCommand> = saga();
/// let order_created_event = OrderEvent::Created(OrderCreatedEvent {
///         order_id: 1,
///         customer_name: "John Doe".to_string(),
///         items: vec!["Item 1".to_string(), "Item 2".to_string()],
///     });
///
/// let commands = (saga.react)(&order_created_event);
/// ```
pub struct Saga<'a, AR: 'a, A: 'a> {
    /// The `react` function is driving the next action based on the action result.
    pub react: ReactFunction<'a, AR, A>,
}

impl<'a, AR, A> Saga<'a, AR, A> {
    /// Maps the Saga over the A/Action type parameter.
    /// Creates a new instance of [Saga]`<AR, A2>`.
    pub fn map_action<A2, F>(self, f: &'a F) -> Saga<'a, AR, A2>
    where
        F: Fn(&A) -> A2 + Send + Sync,
    {
        let new_react = Box::new(move |ar: &AR| {
            let a = (self.react)(ar);
            a.into_iter().map(|a: A| f(&a)).collect()
        });

        Saga { react: new_react }
    }

    /// Maps the Saga over the AR/ActionResult type parameter.
    /// Creates a new instance of [Saga]`<AR2, A>`.
    pub fn map_action_result<AR2, F>(self, f: &'a F) -> Saga<'a, AR2, A>
    where
        F: Fn(&AR2) -> AR + Send + Sync,
    {
        let new_react = Box::new(move |ar2: &AR2| {
            let ar = f(ar2);
            (self.react)(&ar)
        });

        Saga { react: new_react }
    }
}
