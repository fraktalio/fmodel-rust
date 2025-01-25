use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

use fmodel_rust::aggregate::{
    EventRepository, EventSourcedAggregate, EventSourcedOrchestratingAggregate, StateRepository,
    StateStoredAggregate, StateStoredOrchestratingAggregate,
};
use fmodel_rust::decider::Decider;
use fmodel_rust::saga::Saga;
use fmodel_rust::Identifier;

use crate::api::{
    CancelOrderCommand, CreateOrderCommand, CreateShipmentCommand, OrderCancelledEvent,
    OrderCommand, OrderCreatedEvent, OrderEvent, OrderState, OrderUpdatedEvent, ShipmentCommand,
    ShipmentCreatedEvent, ShipmentEvent, ShipmentState, UpdateOrderCommand,
};
use crate::application::{
    command_from_sum, event_from_sum, sum_to_command, sum_to_event, AggregateError, Command, Event,
};

mod api;
mod application;

/// A simple in-memory event repository - infrastructure
struct InMemoryEventRepository {
    events: RwLock<Vec<(Event, i32)>>,
}

impl InMemoryEventRepository {
    fn new() -> Self {
        InMemoryEventRepository {
            events: RwLock::new(vec![]),
        }
    }
}

/// Implementation of [EventRepository] for [InMemoryEventRepository] - infrastructure
impl EventRepository<Command, Event, i32, AggregateError> for InMemoryEventRepository {
    async fn fetch_events(&self, command: &Command) -> Result<Vec<(Event, i32)>, AggregateError> {
        Ok(self
            .events
            .read()
            .unwrap()
            .clone()
            .into_iter()
            .filter(|(event, _)| event.identifier() == command.identifier())
            .collect())
    }

    async fn save(&self, events: &[Event]) -> Result<Vec<(Event, i32)>, AggregateError> {
        let mut latest_version = self
            .version_provider(events.first().unwrap())
            .await?
            .unwrap_or(-1);
        let events = events
            .iter()
            .map(|event| {
                latest_version += 1;
                (event.clone(), latest_version)
            })
            .collect::<Vec<(Event, i32)>>();

        self.events
            .write()
            .unwrap()
            .extend_from_slice(&events.clone());
        Ok(events)
    }

    async fn version_provider(&self, event: &Event) -> Result<Option<i32>, AggregateError> {
        Ok(self
            .events
            .read()
            .unwrap()
            .clone()
            .into_iter()
            .filter(|(e, _)| e.identifier() == event.identifier())
            .map(|(_, version)| version)
            .last())
    }
}

#[allow(clippy::type_complexity)]
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
impl StateRepository<Command, (OrderState, ShipmentState), i32, AggregateError>
    for InMemoryStateRepository
{
    async fn fetch_state(
        &self,
        command: &Command,
    ) -> Result<Option<((OrderState, ShipmentState), i32)>, AggregateError> {
        Ok(self
            .states
            .lock()
            .unwrap()
            .get(&command.identifier().parse::<u32>().unwrap())
            .cloned())
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
            .insert(state.0.order_id, (state.clone(), version + 1));
        Ok((state.clone(), version))
    }
}

/// Decider for the Order aggregate - Domain logic
fn order_decider<'a>() -> Decider<'a, OrderCommand, OrderState, OrderEvent> {
    Decider {
        decide: Box::new(|command, state| match command {
            OrderCommand::Create(cmd) => Ok(vec![OrderEvent::Created(OrderCreatedEvent {
                order_id: cmd.order_id,
                customer_name: cmd.customer_name.to_owned(),
                items: cmd.items.to_owned(),
            })]),
            OrderCommand::Update(cmd) => Ok(vec![OrderEvent::Updated(OrderUpdatedEvent {
                order_id: cmd.order_id,
                updated_items: cmd.new_items.to_owned(),
            })]),
            OrderCommand::Cancel(cmd) => {
                if state.order_id == cmd.order_id {
                    Ok(vec![OrderEvent::Cancelled(OrderCancelledEvent {
                        order_id: cmd.order_id,
                    })])
                } else {
                    Ok(vec![])
                }
            }
        }),
        evolve: Box::new(|state, event| {
            let mut new_state = state.clone();
            match event {
                OrderEvent::Created(evt) => {
                    new_state.order_id = evt.order_id;
                    new_state.customer_name = evt.customer_name.to_owned();
                    new_state.items = evt.items.to_owned();
                }
                OrderEvent::Updated(evt) => {
                    if new_state.order_id == evt.order_id {
                        new_state.items = evt.updated_items.to_owned();
                    }
                }
                OrderEvent::Cancelled(evt) => {
                    if new_state.order_id == evt.order_id {
                        new_state.is_cancelled = true;
                    }
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
            ShipmentCommand::Create(cmd) => {
                Ok(vec![ShipmentEvent::Created(ShipmentCreatedEvent {
                    shipment_id: cmd.shipment_id,
                    order_id: cmd.order_id,
                    customer_name: cmd.customer_name.to_owned(),
                    items: cmd.items.to_owned(),
                })])
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

fn order_saga<'a>() -> Saga<'a, OrderEvent, ShipmentCommand> {
    Saga {
        react: Box::new(|event| match event {
            OrderEvent::Created(evt) => {
                vec![ShipmentCommand::Create(CreateShipmentCommand {
                    shipment_id: evt.order_id,
                    order_id: evt.order_id,
                    customer_name: evt.customer_name.to_owned(),
                    items: evt.items.to_owned(),
                })]
            }
            OrderEvent::Updated(_) => {
                vec![]
            }
            OrderEvent::Cancelled(_) => {
                vec![]
            }
        }),
    }
}

fn shipment_saga<'a>() -> Saga<'a, ShipmentEvent, OrderCommand> {
    Saga {
        react: Box::new(|event| match event {
            ShipmentEvent::Created(evt) => {
                vec![OrderCommand::Update(api::UpdateOrderCommand {
                    order_id: evt.order_id,
                    new_items: evt.items.to_owned(),
                })]
            }
        }),
    }
}

#[tokio::test]
async fn event_sourced_aggregate_test() {
    let combined_decider = order_decider()
        .combine(shipment_decider()) // Decider<Sum<OrderCommand, ShipmentCommand>, (OrderState, ShipmentState), Sum<OrderEvent, ShipmentEvent>>
        .map_command(&command_from_sum) // Decider<Command, (OrderState, ShipmentState), Sum<OrderEvent, ShipmentEvent>>
        .map_event(&event_from_sum, &sum_to_event); // Decider<Command, (OrderState, ShipmentState), Event>
    let repository = InMemoryEventRepository::new();
    let aggregate = Arc::new(EventSourcedAggregate::new(
        repository,
        combined_decider.map_error(&|()| AggregateError::DomainError("Decider error".to_string())),
    ));
    // Makes a clone of the Arc pointer.
    // This creates another pointer to the same allocation, increasing the strong reference count.
    let aggregate2 = Arc::clone(&aggregate);

    // Let's spawn two threads to simulate two concurrent requests
    let handle1 = thread::spawn(|| async move {
        let command = Command::OrderCreate(CreateOrderCommand {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });

        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Event::OrderCreated(OrderCreatedEvent {
                    order_id: 1,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 1".to_string(), "Item 2".to_string()],
                }),
                0
            )]
        );
        let command = Command::OrderUpdate(UpdateOrderCommand {
            order_id: 1,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Event::OrderUpdated(OrderUpdatedEvent {
                    order_id: 1,
                    updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
                }),
                1
            )]
        );
        let command = Command::OrderCancel(CancelOrderCommand { order_id: 1 });
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Event::OrderCancelled(OrderCancelledEvent { order_id: 1 }),
                2
            )]
        );
    });

    let handle2 = thread::spawn(|| async move {
        let command = Command::OrderCreate(CreateOrderCommand {
            order_id: 2,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Event::OrderCreated(OrderCreatedEvent {
                    order_id: 2,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 1".to_string(), "Item 2".to_string()],
                }),
                0
            )]
        );
        let command = Command::OrderUpdate(UpdateOrderCommand {
            order_id: 2,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Event::OrderUpdated(OrderUpdatedEvent {
                    order_id: 2,
                    updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
                }),
                1
            )]
        );
        let command = Command::OrderCancel(CancelOrderCommand { order_id: 2 });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Event::OrderCancelled(OrderCancelledEvent { order_id: 2 }),
                2
            )]
        );
    });

    handle1.join().unwrap().await;
    handle2.join().unwrap().await;
}

#[tokio::test]
async fn orchestrated_event_sourced_aggregate_test() {
    let combined_decider = order_decider()
        .combine(shipment_decider()) // Decider<Sum<OrderCommand, ShipmentCommand>, (OrderState, ShipmentState), Sum<OrderEvent, ShipmentEvent>>
        .map_command(&command_from_sum) // Decider<Command, (OrderState, ShipmentState), Sum<OrderEvent, ShipmentEvent>>
        .map_event(&event_from_sum, &sum_to_event); // Decider<Command, (OrderState, ShipmentState), Event>
    let combined_saga = order_saga()
        .combine(shipment_saga())
        .map_action(&sum_to_command)
        .map_action_result(&event_from_sum);
    let repository = InMemoryEventRepository::new();
    let aggregate = Arc::new(EventSourcedOrchestratingAggregate::new(
        repository,
        combined_decider.map_error(&|()| AggregateError::DomainError("Decider error".to_string())),
        combined_saga,
    ));
    // Makes a clone of the Arc pointer.
    // This creates another pointer to the same allocation, increasing the strong reference count.
    let aggregate2 = Arc::clone(&aggregate);

    // Let's spawn two threads to simulate two concurrent requests
    let handle1 = thread::spawn(|| async move {
        let command = Command::OrderCreate(CreateOrderCommand {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });

        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [
                (
                    Event::OrderCreated(OrderCreatedEvent {
                        order_id: 1,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    }),
                    0
                ),
                (
                    Event::ShipmentCreated(ShipmentCreatedEvent {
                        shipment_id: 1,
                        order_id: 1,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    }),
                    1
                ),
                (
                    Event::OrderUpdated(OrderUpdatedEvent {
                        order_id: 1,
                        updated_items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    }),
                    2
                ),
            ]
        );
        let command = Command::OrderUpdate(UpdateOrderCommand {
            order_id: 1,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Event::OrderUpdated(OrderUpdatedEvent {
                    order_id: 1,
                    updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
                }),
                3
            )]
        );
        let command = Command::OrderCancel(CancelOrderCommand { order_id: 1 });
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Event::OrderCancelled(OrderCancelledEvent { order_id: 1 }),
                4
            )]
        );
    });

    let handle2 = thread::spawn(|| async move {
        let command = Command::OrderCreate(CreateOrderCommand {
            order_id: 2,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [
                (
                    Event::OrderCreated(OrderCreatedEvent {
                        order_id: 2,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    }),
                    0
                ),
                (
                    Event::ShipmentCreated(ShipmentCreatedEvent {
                        shipment_id: 2,
                        order_id: 2,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    }),
                    1
                ),
                (
                    Event::OrderUpdated(OrderUpdatedEvent {
                        order_id: 2,
                        updated_items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    }),
                    2
                ),
            ]
        );
        let command = Command::OrderUpdate(UpdateOrderCommand {
            order_id: 2,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Event::OrderUpdated(OrderUpdatedEvent {
                    order_id: 2,
                    updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
                }),
                3
            )]
        );
        let command = Command::OrderCancel(CancelOrderCommand { order_id: 2 });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                Event::OrderCancelled(OrderCancelledEvent { order_id: 2 }),
                4
            )]
        );
    });

    handle1.join().unwrap().await;
    handle2.join().unwrap().await;
}

#[tokio::test]
async fn state_stored_aggregate_test() {
    let combined_decider = order_decider()
        .combine(shipment_decider()) // Decider<Sum<OrderCommand, ShipmentCommand>, (OrderState, ShipmentState), Sum<OrderEvent, ShipmentEvent>>
        .map_command(&command_from_sum) // Decider<Command, (OrderState, ShipmentState), Sum<OrderEvent, ShipmentEvent>>
        .map_event(&event_from_sum, &sum_to_event); // Decider<Command, (OrderState, ShipmentState), Event>

    let repository = InMemoryStateRepository::new();
    let aggregate = Arc::new(StateStoredAggregate::new(
        repository,
        combined_decider.map_error(&|()| AggregateError::DomainError("Decider error".to_string())),
    ));
    let aggregate2 = Arc::clone(&aggregate);

    let handle1 = thread::spawn(|| async move {
        let command = Command::OrderCreate(CreateOrderCommand {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
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
        let command = Command::OrderUpdate(UpdateOrderCommand {
            order_id: 1,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
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
        let command = Command::OrderCancel(CancelOrderCommand { order_id: 1 });
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
        let command = Command::OrderCreate(CreateOrderCommand {
            order_id: 2,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
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
        let command = Command::OrderUpdate(UpdateOrderCommand {
            order_id: 2,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
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
        let command = Command::OrderCancel(CancelOrderCommand { order_id: 2 });
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

#[tokio::test]
async fn state_stored_combined_test() {
    let combined_decider = order_decider()
        .combine(shipment_decider()) // Decider<Sum<OrderCommand, ShipmentCommand>, (OrderState, ShipmentState), Sum<OrderEvent, ShipmentEvent>>
        .map_command(&command_from_sum) // Decider<Command, (OrderState, ShipmentState), Sum<OrderEvent, ShipmentEvent>>
        .map_event(&event_from_sum, &sum_to_event); // Decider<Command, (OrderState, ShipmentState), Event>

    let combined_saga = order_saga()
        .combine(shipment_saga())
        .map_action(&sum_to_command)
        .map_action_result(&event_from_sum);

    let repository = InMemoryStateRepository::new();
    let aggregate = Arc::new(StateStoredOrchestratingAggregate::new(
        repository,
        combined_decider.map_error(&|()| AggregateError::DomainError("Decider error".to_string())),
        combined_saga,
    ));
    let aggregate2 = Arc::clone(&aggregate);

    let handle1 = thread::spawn(|| async move {
        let command = Command::OrderCreate(CreateOrderCommand {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
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
                        shipment_id: 1,
                        order_id: 1,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    }
                ),
                0
            )
        );
        let command = Command::OrderUpdate(UpdateOrderCommand {
            order_id: 1,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
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
                        shipment_id: 1,
                        order_id: 1,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    }
                ),
                1
            )
        );
        let command = Command::OrderCancel(CancelOrderCommand { order_id: 1 });
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
                        shipment_id: 1,
                        order_id: 1,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    }
                ),
                2
            )
        );
    });

    let handle2 = thread::spawn(|| async move {
        let command = Command::OrderCreate(CreateOrderCommand {
            order_id: 2,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
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
                        shipment_id: 2,
                        order_id: 2,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    }
                ),
                0
            )
        );
        let command = Command::OrderUpdate(UpdateOrderCommand {
            order_id: 2,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
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
                        shipment_id: 2,
                        order_id: 2,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    }
                ),
                1
            )
        );
        let command = Command::OrderCancel(CancelOrderCommand { order_id: 2 });
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
                        shipment_id: 2,
                        order_id: 2,
                        customer_name: "John Doe".to_string(),
                        items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    }
                ),
                2
            )
        );
    });

    handle1.join().unwrap().await;
    handle2.join().unwrap().await;
}
