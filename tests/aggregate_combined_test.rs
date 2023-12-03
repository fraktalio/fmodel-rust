use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;

use async_trait::async_trait;
use derive_more::Display;

use fmodel_rust::aggregate::{
    EventRepository, EventSourcedAggregate, StateRepository, StateStoredAggregate,
};
use fmodel_rust::decider::Decider;
use fmodel_rust::decider_combined::combine;
use fmodel_rust::Sum;

use crate::api::{
    CancelOrderCommand, CreateOrderCommand, OrderCancelledEvent, OrderCommand, OrderCreatedEvent,
    OrderEvent, OrderUpdatedEvent, ShipmentCommand, ShipmentCreatedEvent, ShipmentEvent,
    UpdateOrderCommand,
};

mod api;

/// Error type for the application/aggregate
#[derive(Debug, Display)]
#[allow(dead_code)]
enum AggregateError {
    FetchEvents(String),
    SaveEvents(String),
    FetchState(String),
    SaveState(String),
}

impl Error for AggregateError {}

/// A simple in-memory event repository - infrastructure
struct InMemoryEventRepository {
    events: Mutex<Vec<(Sum<OrderEvent, ShipmentEvent>, i32)>>,
}

impl InMemoryEventRepository {
    fn new() -> Self {
        InMemoryEventRepository {
            events: Mutex::new(vec![]),
        }
    }
}

trait Id {
    fn id(&self) -> u32;
}

impl Id for Sum<OrderEvent, ShipmentEvent> {
    fn id(&self) -> u32 {
        match self {
            Sum::First(event) => event.id(),
            Sum::Second(event) => event.id(),
        }
    }
}

impl Id for Sum<OrderCommand, ShipmentCommand> {
    fn id(&self) -> u32 {
        match self {
            Sum::First(command) => command.id(),
            Sum::Second(command) => command.id(),
        }
    }
}

impl Id for (OrderState, ShipmentState) {
    fn id(&self) -> u32 {
        self.0.order_id
    }
}
/// Implementation of [EventRepository] for [InMemoryEventRepository] - infrastructure
#[async_trait]
impl
    EventRepository<
        Sum<OrderCommand, ShipmentCommand>,
        Sum<OrderEvent, ShipmentEvent>,
        i32,
        AggregateError,
    > for InMemoryEventRepository
{
    async fn fetch_events(
        &self,
        command: &Sum<OrderCommand, ShipmentCommand>,
    ) -> Result<Vec<(Sum<OrderEvent, ShipmentEvent>, i32)>, AggregateError> {
        Ok(self
            .events
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .filter(|(event, _)| event.id() == command.id())
            .collect())
    }

    async fn save(
        &self,
        events: &[Sum<OrderEvent, ShipmentEvent>],
        latest_version: &Option<i32>,
    ) -> Result<Vec<(Sum<OrderEvent, ShipmentEvent>, i32)>, AggregateError> {
        let mut latest_version = latest_version.to_owned().unwrap_or(-1);
        let events = events
            .into_iter()
            .map(|event| {
                latest_version += 1;
                (event.clone(), latest_version)
            })
            .collect::<Vec<(Sum<OrderEvent, ShipmentEvent>, i32)>>();

        self.events
            .lock()
            .unwrap()
            .extend_from_slice(&*events.clone());
        Ok(Vec::from(events))
    }
}

struct InMemoryStateRepository {
    states: Mutex<HashMap<u32, ((OrderState, ShipmentState), i32)>>,
}

impl InMemoryStateRepository {
    fn new() -> Self {
        InMemoryStateRepository {
            states: Mutex::new(HashMap::new()),
        }
    }
}

// Implementation of [StateRepository] for [InMemoryOrderStateRepository]
#[async_trait]
impl
    StateRepository<
        Sum<OrderCommand, ShipmentCommand>,
        (OrderState, ShipmentState),
        i32,
        AggregateError,
    > for InMemoryStateRepository
{
    async fn fetch_state(
        &self,
        command: &Sum<OrderCommand, ShipmentCommand>,
    ) -> Result<Option<((OrderState, ShipmentState), i32)>, AggregateError> {
        Ok(self.states.lock().unwrap().get(&command.id()).cloned())
    }

    async fn save(
        &self,
        state: &(OrderState, ShipmentState),
        version: &Option<i32>,
    ) -> Result<((OrderState, ShipmentState), i32), AggregateError> {
        let version = version.to_owned().unwrap_or(0);
        self.states
            .lock()
            .unwrap()
            .insert(state.id(), (state.clone(), version + 1));
        Ok((state.clone(), version))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct OrderState {
    order_id: u32,
    customer_name: String,
    items: Vec<String>,
    is_cancelled: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct ShipmentState {
    shipment_id: u32,
    order_id: u32,
    customer_name: String,
    items: Vec<String>,
}

/// Decider for the Order aggregate - Domain logic
fn order_decider<'a>() -> Decider<'a, OrderCommand, OrderState, OrderEvent> {
    Decider {
        decide: Box::new(|command, state| match command {
            OrderCommand::Create(create_cmd) => {
                vec![OrderEvent::Created(OrderCreatedEvent {
                    order_id: create_cmd.order_id,
                    customer_name: create_cmd.customer_name.to_owned(),
                    items: create_cmd.items.to_owned(),
                })]
            }
            OrderCommand::Update(update_cmd) => {
                if state.order_id == update_cmd.order_id {
                    vec![OrderEvent::Updated(OrderUpdatedEvent {
                        order_id: update_cmd.order_id,
                        updated_items: update_cmd.new_items.to_owned(),
                    })]
                } else {
                    vec![]
                }
            }
            OrderCommand::Cancel(cancel_cmd) => {
                if state.order_id == cancel_cmd.order_id {
                    vec![OrderEvent::Cancelled(OrderCancelledEvent {
                        order_id: cancel_cmd.order_id,
                    })]
                } else {
                    vec![]
                }
            }
        }),
        evolve: Box::new(|state, event| {
            let mut new_state = state.clone();
            match event {
                OrderEvent::Created(created_event) => {
                    new_state.order_id = created_event.order_id;
                    new_state.customer_name = created_event.customer_name.to_owned();
                    new_state.items = created_event.items.to_owned();
                }
                OrderEvent::Updated(updated_event) => {
                    new_state.items = updated_event.updated_items.to_owned();
                }
                OrderEvent::Cancelled(_) => {
                    new_state.is_cancelled = true;
                }
            }
            new_state
        }),
        initial_state: Box::new(|| OrderState {
            order_id: 0,
            customer_name: "".to_string(),
            items: Vec::new(),
            is_cancelled: false,
        }),
    }
}

/// Decider for the Shipment aggregate - Domain logic
fn shipment_decider<'a>() -> Decider<'a, ShipmentCommand, ShipmentState, ShipmentEvent> {
    Decider {
        decide: Box::new(|command, _state| match command {
            ShipmentCommand::Create(create_cmd) => {
                vec![ShipmentEvent::Created(ShipmentCreatedEvent {
                    shipment_id: create_cmd.shipment_id,
                    order_id: create_cmd.order_id,
                    customer_name: create_cmd.customer_name.to_owned(),
                    items: create_cmd.items.to_owned(),
                })]
            }
        }),
        evolve: Box::new(|state, event| {
            let mut new_state = state.clone();
            match event {
                ShipmentEvent::Created(created_event) => {
                    new_state.shipment_id = created_event.shipment_id;
                    new_state.order_id = created_event.order_id;
                    new_state.customer_name = created_event.customer_name.to_owned();
                    new_state.items = created_event.items.to_owned();
                }
            }
            new_state
        }),
        initial_state: Box::new(|| ShipmentState {
            shipment_id: 0,
            order_id: 0,
            customer_name: "".to_string(),
            items: Vec::new(),
        }),
    }
}

#[tokio::test]
async fn es_test() {
    let combined_decider = combine(order_decider(), shipment_decider());
    let repository = InMemoryEventRepository::new();
    let aggregate = Arc::new(EventSourcedAggregate::new(repository, combined_decider));
    // Makes a clone of the Arc pointer.
    // This creates another pointer to the same allocation, increasing the strong reference count.
    let aggregate2 = Arc::clone(&aggregate);

    // Lets spawn two threads to simulate two concurrent requests
    let handle1 = thread::spawn(|| async move {
        let command = Sum::First(OrderCommand::Create(CreateOrderCommand {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        }));

        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Sum::First(OrderEvent::Created(OrderCreatedEvent {
                    order_id: 1,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 1".to_string(), "Item 2".to_string()],
                })),
                0
            )]
        );
        let command = Sum::First(OrderCommand::Update(UpdateOrderCommand {
            order_id: 1,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        }));
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Sum::First(OrderEvent::Updated(OrderUpdatedEvent {
                    order_id: 1,
                    updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
                })),
                1
            )]
        );
        let command = Sum::First(OrderCommand::Cancel(CancelOrderCommand { order_id: 1 }));
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Sum::First(OrderEvent::Cancelled(OrderCancelledEvent { order_id: 1 })),
                2
            )]
        );
    });

    let handle2 = thread::spawn(|| async move {
        let command = Sum::First(OrderCommand::Create(CreateOrderCommand {
            order_id: 2,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        }));
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Sum::First(OrderEvent::Created(OrderCreatedEvent {
                    order_id: 2,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 1".to_string(), "Item 2".to_string()],
                })),
                0
            )]
        );
        let command = Sum::First(OrderCommand::Update(UpdateOrderCommand {
            order_id: 2,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        }));
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Sum::First(OrderEvent::Updated(OrderUpdatedEvent {
                    order_id: 2,
                    updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
                })),
                1
            )]
        );
        let command = Sum::First(OrderCommand::Cancel(CancelOrderCommand { order_id: 2 }));
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Sum::First(OrderEvent::Cancelled(OrderCancelledEvent { order_id: 2 })),
                2
            )]
        );
    });

    handle1.join().unwrap().await;
    handle2.join().unwrap().await;
}

#[tokio::test]
async fn ss_test() {
    let combined_decider = combine(order_decider(), shipment_decider());
    let repository = InMemoryStateRepository::new();
    let aggregate = Arc::new(StateStoredAggregate::new(repository, combined_decider));
    let aggregate2 = Arc::clone(&aggregate);

    let handle1 = thread::spawn(|| async move {
        let command = Sum::First(OrderCommand::Create(CreateOrderCommand {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        }));
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                (
                    OrderState {
                        order_id: 1,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 1".to_string(), "Item 2".to_string()],
                        is_cancelled: false,
                    },
                    ShipmentState {
                        shipment_id: 0,
                        order_id: 0,
                        customer_name: "".to_string(),
                        items: Vec::new(),
                    }
                ),
                0
            )
        );
        let command = Sum::First(OrderCommand::Update(UpdateOrderCommand {
            order_id: 1,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        }));
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                (
                    OrderState {
                        order_id: 1,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 3".to_string(), "Item 4".to_string()],
                        is_cancelled: false,
                    },
                    ShipmentState {
                        shipment_id: 0,
                        order_id: 0,
                        customer_name: "".to_string(),
                        items: Vec::new(),
                    }
                ),
                1
            )
        );
        let command = Sum::First(OrderCommand::Cancel(CancelOrderCommand { order_id: 1 }));
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                (
                    OrderState {
                        order_id: 1,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 3".to_string(), "Item 4".to_string()],
                        is_cancelled: true,
                    },
                    ShipmentState {
                        shipment_id: 0,
                        order_id: 0,
                        customer_name: "".to_string(),
                        items: Vec::new(),
                    }
                ),
                2
            )
        );
    });

    let handle2 = thread::spawn(|| async move {
        let command = Sum::First(OrderCommand::Create(CreateOrderCommand {
            order_id: 2,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        }));
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                (
                    OrderState {
                        order_id: 2,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 1".to_string(), "Item 2".to_string()],
                        is_cancelled: false,
                    },
                    ShipmentState {
                        shipment_id: 0,
                        order_id: 0,
                        customer_name: "".to_string(),
                        items: Vec::new(),
                    }
                ),
                0
            )
        );
        let command = Sum::First(OrderCommand::Update(UpdateOrderCommand {
            order_id: 2,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        }));
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                (
                    OrderState {
                        order_id: 2,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 3".to_string(), "Item 4".to_string()],
                        is_cancelled: false,
                    },
                    ShipmentState {
                        shipment_id: 0,
                        order_id: 0,
                        customer_name: "".to_string(),
                        items: Vec::new(),
                    }
                ),
                1
            )
        );
        let command = Sum::First(OrderCommand::Cancel(CancelOrderCommand { order_id: 2 }));
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                (
                    OrderState {
                        order_id: 2,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 3".to_string(), "Item 4".to_string()],
                        is_cancelled: true,
                    },
                    ShipmentState {
                        shipment_id: 0,
                        order_id: 0,
                        customer_name: "".to_string(),
                        items: Vec::new(),
                    }
                ),
                2
            )
        );
    });

    handle1.join().unwrap().await;
    handle2.join().unwrap().await;
}
