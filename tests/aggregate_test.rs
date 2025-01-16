use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use fmodel_rust::aggregate::{
    EventRepository, EventSourcedAggregate, StateRepository, StateStoredAggregate,
};
use fmodel_rust::decider::Decider;

use crate::api::{
    CancelOrderCommand, CreateOrderCommand, OrderCancelledEvent, OrderCommand, OrderCreatedEvent,
    OrderEvent, OrderState, OrderUpdatedEvent, UpdateOrderCommand,
};
use crate::application::AggregateError;

mod api;
mod application;

/// A simple in-memory event repository - infrastructure
struct InMemoryOrderEventRepository {
    events: Mutex<Vec<(OrderEvent, i32)>>,
}

impl InMemoryOrderEventRepository {
    fn new() -> Self {
        InMemoryOrderEventRepository {
            events: Mutex::new(vec![]),
        }
    }
}

/// Implementation of [EventRepository] for [InMemoryOrderEventRepository] - infrastructure
impl EventRepository<OrderCommand, OrderEvent, i32, AggregateError>
    for InMemoryOrderEventRepository
{
    async fn fetch_events(
        &self,
        command: &OrderCommand,
    ) -> Result<Vec<(OrderEvent, i32)>, AggregateError> {
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
        events: &[OrderEvent],
        latest_version: &Option<i32>,
    ) -> Result<Vec<(OrderEvent, i32)>, AggregateError> {
        let mut latest_version = latest_version.to_owned().unwrap_or(-1);
        let events = events
            .iter()
            .map(|event| {
                latest_version += 1;
                (event.clone(), latest_version)
            })
            .collect::<Vec<(OrderEvent, i32)>>();

        self.events
            .lock()
            .unwrap()
            .extend_from_slice(&events.clone());
        Ok(events)
    }
}

struct InMemoryOrderStateRepository {
    states: Mutex<HashMap<u32, (OrderState, i32)>>,
}

impl InMemoryOrderStateRepository {
    fn new() -> Self {
        InMemoryOrderStateRepository {
            states: Mutex::new(HashMap::new()),
        }
    }
}

// Implementation of [StateRepository] for [InMemoryOrderStateRepository]
impl StateRepository<OrderCommand, OrderState, i32, AggregateError>
    for InMemoryOrderStateRepository
{
    async fn fetch_state(
        &self,
        command: &OrderCommand,
    ) -> Result<Option<(OrderState, i32)>, AggregateError> {
        Ok(self.states.lock().unwrap().get(&command.id()).cloned())
    }

    async fn save(
        &self,
        state: &OrderState,
        version: &Option<i32>,
    ) -> Result<(OrderState, i32), AggregateError> {
        let version = version.to_owned().unwrap_or(0);
        self.states
            .lock()
            .unwrap()
            .insert(state.order_id, (state.clone(), version + 1));
        Ok((state.clone(), version))
    }
}

/// Decider for the Order aggregate - Domain logic
fn decider<'a>() -> Decider<'a, OrderCommand, OrderState, OrderEvent> {
    Decider {
        decide: Box::new(|command, state| match command {
            OrderCommand::Create(cmd) => Ok(vec![OrderEvent::Created(OrderCreatedEvent {
                order_id: cmd.order_id,
                customer_name: cmd.customer_name.to_owned(),
                items: cmd.items.to_owned(),
            })]),
            OrderCommand::Update(cmd) => {
                if state.order_id == cmd.order_id {
                    Ok(vec![OrderEvent::Updated(OrderUpdatedEvent {
                        order_id: cmd.order_id,
                        updated_items: cmd.new_items.to_owned(),
                    })])
                } else {
                    Ok(vec![])
                }
            }
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
                    new_state.items = evt.updated_items.to_owned();
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

#[tokio::test]
async fn es_test() {
    let repository = InMemoryOrderEventRepository::new();
    let aggregate = Arc::new(EventSourcedAggregate::new(
        repository,
        decider().map_error(&|()| AggregateError::DomainError("Decider error".to_string())),
    ));
    // Makes a clone of the Arc pointer.
    // This creates another pointer to the same allocation, increasing the strong reference count.
    let aggregate2 = Arc::clone(&aggregate);

    // Let's spawn two threads to simulate two concurrent requests
    let handle1 = thread::spawn(|| async move {
        let command = OrderCommand::Create(CreateOrderCommand {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });

        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                OrderEvent::Created(OrderCreatedEvent {
                    order_id: 1,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 1".to_string(), "Item 2".to_string()],
                }),
                0
            )]
        );
        let command = OrderCommand::Update(UpdateOrderCommand {
            order_id: 1,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                OrderEvent::Updated(OrderUpdatedEvent {
                    order_id: 1,
                    updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
                }),
                1
            )]
        );
        let command = OrderCommand::Cancel(CancelOrderCommand { order_id: 1 });
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                OrderEvent::Cancelled(OrderCancelledEvent { order_id: 1 }),
                2
            )]
        );
    });

    let handle2 = thread::spawn(|| async move {
        let command = OrderCommand::Create(CreateOrderCommand {
            order_id: 2,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                OrderEvent::Created(OrderCreatedEvent {
                    order_id: 2,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 1".to_string(), "Item 2".to_string()],
                }),
                0
            )]
        );
        let command = OrderCommand::Update(UpdateOrderCommand {
            order_id: 2,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                OrderEvent::Updated(OrderUpdatedEvent {
                    order_id: 2,
                    updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
                }),
                1
            )]
        );
        let command = OrderCommand::Cancel(CancelOrderCommand { order_id: 2 });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [(
                OrderEvent::Cancelled(OrderCancelledEvent { order_id: 2 }),
                2
            )]
        );
    });

    handle1.join().unwrap().await;
    handle2.join().unwrap().await;
}

#[tokio::test]
async fn ss_test() {
    let repository = InMemoryOrderStateRepository::new();
    let aggregate = Arc::new(StateStoredAggregate::new(
        repository,
        decider().map_error(&|()| AggregateError::DomainError("Decider error".to_string())),
    ));
    let aggregate2 = Arc::clone(&aggregate);

    let handle1 = thread::spawn(|| async move {
        let command = OrderCommand::Create(CreateOrderCommand {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                OrderState {
                    order_id: 1,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    is_cancelled: false,
                },
                0
            )
        );
        let command = OrderCommand::Update(UpdateOrderCommand {
            order_id: 1,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                OrderState {
                    order_id: 1,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 3".to_string(), "Item 4".to_string()],
                    is_cancelled: false,
                },
                1
            )
        );
        let command = OrderCommand::Cancel(CancelOrderCommand { order_id: 1 });
        let result = aggregate.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                OrderState {
                    order_id: 1,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 3".to_string(), "Item 4".to_string()],
                    is_cancelled: true,
                },
                2
            )
        );
    });

    let handle2 = thread::spawn(|| async move {
        let command = OrderCommand::Create(CreateOrderCommand {
            order_id: 2,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                OrderState {
                    order_id: 2,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    is_cancelled: false,
                },
                0
            )
        );
        let command = OrderCommand::Update(UpdateOrderCommand {
            order_id: 2,
            new_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                OrderState {
                    order_id: 2,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 3".to_string(), "Item 4".to_string()],
                    is_cancelled: false,
                },
                1
            )
        );
        let command = OrderCommand::Cancel(CancelOrderCommand { order_id: 2 });
        let result = aggregate2.handle(&command).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                OrderState {
                    order_id: 2,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 3".to_string(), "Item 4".to_string()],
                    is_cancelled: true,
                },
                2
            )
        );
    });

    handle1.join().unwrap().await;
    handle2.join().unwrap().await;
}
