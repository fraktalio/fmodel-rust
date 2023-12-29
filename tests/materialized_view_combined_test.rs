use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;

use derive_more::Display;

use fmodel_rust::materialized_view::{MaterializedView, ViewStateRepository};
use fmodel_rust::view::View;
use fmodel_rust::Sum;

use crate::api::{
    OrderCancelledEvent, OrderCreatedEvent, OrderEvent, OrderUpdatedEvent, ShipmentEvent,
};

mod api;

#[derive(Debug, Clone, PartialEq)]
struct OrderViewState {
    order_id: u32,
    customer_name: String,
    items: Vec<String>,
    is_cancelled: bool,
}

#[derive(Debug, Clone, PartialEq)]
struct ShipmentViewState {
    shipment_id: u32,
    order_id: u32,
    customer_name: String,
    items: Vec<String>,
}

fn order_view<'a>() -> View<'a, OrderViewState, OrderEvent> {
    View {
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
        initial_state: Box::new(|| OrderViewState {
            order_id: 0,
            customer_name: "".to_string(),
            items: Vec::new(),
            is_cancelled: false,
        }),
    }
}

fn shipment_view<'a>() -> View<'a, ShipmentViewState, ShipmentEvent> {
    View {
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
        initial_state: Box::new(|| ShipmentViewState {
            shipment_id: 0,
            order_id: 0,
            customer_name: "".to_string(),
            items: Vec::new(),
        }),
    }
}

/// Error type for the application/materialized view
#[derive(Debug, Display)]
#[allow(dead_code)]
enum MaterializedViewError {
    FetchState(String),
    SaveState(String),
}

impl Error for MaterializedViewError {}

struct InMemoryViewStateRepository {
    states: Mutex<HashMap<u32, (OrderViewState, ShipmentViewState)>>,
}

impl InMemoryViewStateRepository {
    fn new() -> Self {
        InMemoryViewStateRepository {
            states: Mutex::new(HashMap::new()),
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

impl Id for (OrderViewState, ShipmentViewState) {
    fn id(&self) -> u32 {
        self.0.order_id
    }
}

// Implementation of [ViewStateRepository] for [InMemoryViewOrderStateRepository]
impl
    ViewStateRepository<
        Sum<OrderEvent, ShipmentEvent>,
        (OrderViewState, ShipmentViewState),
        MaterializedViewError,
    > for InMemoryViewStateRepository
{
    async fn fetch_state(
        &self,
        event: &Sum<OrderEvent, ShipmentEvent>,
    ) -> Result<Option<(OrderViewState, ShipmentViewState)>, MaterializedViewError> {
        Ok(self.states.lock().unwrap().get(&event.id()).cloned())
    }

    async fn save(
        &self,
        state: &(OrderViewState, ShipmentViewState),
    ) -> Result<(OrderViewState, ShipmentViewState), MaterializedViewError> {
        self.states
            .lock()
            .unwrap()
            .insert(state.id(), state.clone());
        Ok(state.clone())
    }
}

#[tokio::test]
async fn test() {
    let combined_view = order_view().combine(shipment_view());
    let repository = InMemoryViewStateRepository::new();
    let materialized_view = Arc::new(MaterializedView::new(repository, combined_view));
    let materialized_view1 = Arc::clone(&materialized_view);
    let materialized_view2 = Arc::clone(&materialized_view);

    // Lets spawn two threads to simulate two concurrent requests
    let handle1 = thread::spawn(|| async move {
        let event = Sum::First(OrderEvent::Created(OrderCreatedEvent {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        }));
        let result = materialized_view1.handle(&event).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                OrderViewState {
                    order_id: 1,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    is_cancelled: false,
                },
                ShipmentViewState {
                    shipment_id: 0,
                    order_id: 0,
                    customer_name: "".to_string(),
                    items: Vec::new(),
                }
            )
        );
        let event = Sum::First(OrderEvent::Updated(OrderUpdatedEvent {
            order_id: 1,
            updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        }));
        let result = materialized_view1.handle(&event).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                OrderViewState {
                    order_id: 1,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 3".to_string(), "Item 4".to_string()],
                    is_cancelled: false,
                },
                ShipmentViewState {
                    shipment_id: 0,
                    order_id: 0,
                    customer_name: "".to_string(),
                    items: Vec::new(),
                }
            )
        );
        let event = Sum::First(OrderEvent::Cancelled(OrderCancelledEvent { order_id: 1 }));
        let result = materialized_view1.handle(&event).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                OrderViewState {
                    order_id: 1,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 3".to_string(), "Item 4".to_string()],
                    is_cancelled: true,
                },
                ShipmentViewState {
                    shipment_id: 0,
                    order_id: 0,
                    customer_name: "".to_string(),
                    items: Vec::new(),
                }
            )
        );
    });

    let handle2 = thread::spawn(|| async move {
        let event = Sum::First(OrderEvent::Created(OrderCreatedEvent {
            order_id: 2,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        }));
        let result = materialized_view2.handle(&event).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                OrderViewState {
                    order_id: 2,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 1".to_string(), "Item 2".to_string()],
                    is_cancelled: false,
                },
                ShipmentViewState {
                    shipment_id: 0,
                    order_id: 0,
                    customer_name: "".to_string(),
                    items: Vec::new(),
                }
            )
        );
        let event = Sum::First(OrderEvent::Updated(OrderUpdatedEvent {
            order_id: 2,
            updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        }));
        let result = materialized_view2.handle(&event).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                OrderViewState {
                    order_id: 2,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 3".to_string(), "Item 4".to_string()],
                    is_cancelled: false,
                },
                ShipmentViewState {
                    shipment_id: 0,
                    order_id: 0,
                    customer_name: "".to_string(),
                    items: Vec::new(),
                }
            )
        );
        let event = Sum::First(OrderEvent::Cancelled(OrderCancelledEvent { order_id: 2 }));
        let result = materialized_view2.handle(&event).await;
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            (
                OrderViewState {
                    order_id: 2,
                    customer_name: "John Doe".to_string(),
                    items: vec!["Item 3".to_string(), "Item 4".to_string()],
                    is_cancelled: true,
                },
                ShipmentViewState {
                    shipment_id: 0,
                    order_id: 0,
                    customer_name: "".to_string(),
                    items: Vec::new(),
                }
            )
        );
    });

    handle1.join().unwrap().await;
    handle2.join().unwrap().await;
}
