#![cfg(not(feature = "not-send-futures"))]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use fmodel_rust::materialized_view::{MaterializedView, ViewStateRepository};
use fmodel_rust::view::View;
use fmodel_rust::Identifier;

use crate::api::{
    OrderCancelledEvent, OrderCreatedEvent, OrderUpdatedEvent, OrderViewState, ShipmentViewState,
};
use crate::application::{Event, MaterializedViewError};

mod api;
mod application;

fn order_view<'a>() -> View<'a, OrderViewState, Event> {
    View {
        evolve: Box::new(|state, event| {
            let mut new_state = state.clone();
            match event {
                Event::OrderCreated(evt) => {
                    new_state.order_id = evt.order_id;
                    new_state.customer_name = evt.customer_name.to_owned();
                    new_state.items = evt.items.to_owned();
                }
                Event::OrderUpdated(evt) => {
                    new_state.items = evt.updated_items.to_owned();
                }
                Event::OrderCancelled(_) => {
                    new_state.is_cancelled = true;
                }
                Event::ShipmentCreated(_) => {}
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

fn shipment_view<'a>() -> View<'a, ShipmentViewState, Event> {
    View {
        evolve: Box::new(|state, event| {
            let mut new_state = state.clone();
            match event {
                Event::ShipmentCreated(evt) => {
                    new_state.shipment_id = evt.shipment_id;
                    new_state.order_id = evt.order_id;
                    new_state.customer_name = evt.customer_name.to_owned();
                    new_state.items = evt.items.to_owned();
                }
                Event::OrderCreated(_) => {}
                Event::OrderUpdated(_) => {}
                Event::OrderCancelled(_) => {}
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

// Implementation of [ViewStateRepository] for [InMemoryViewOrderStateRepository]
impl ViewStateRepository<Event, (OrderViewState, ShipmentViewState), MaterializedViewError>
    for InMemoryViewStateRepository
{
    async fn fetch_state(
        &self,
        event: &Event,
    ) -> Result<Option<(OrderViewState, ShipmentViewState)>, MaterializedViewError> {
        Ok(self
            .states
            .lock()
            .unwrap()
            .get(&event.identifier().parse::<u32>().unwrap())
            .cloned())
    }

    async fn save(
        &self,
        state: &(OrderViewState, ShipmentViewState),
    ) -> Result<(OrderViewState, ShipmentViewState), MaterializedViewError> {
        self.states
            .lock()
            .unwrap()
            .insert(state.0.order_id, state.clone());
        Ok(state.clone())
    }
}

#[tokio::test]
async fn test() {
    let combined_view = order_view().merge(shipment_view());
    let repository = InMemoryViewStateRepository::new();
    let materialized_view = Arc::new(MaterializedView::new(repository, combined_view));
    let materialized_view1 = Arc::clone(&materialized_view);
    let materialized_view2 = Arc::clone(&materialized_view);

    // Lets spawn two threads to simulate two concurrent requests
    let handle1 = thread::spawn(|| async move {
        let event = Event::OrderCreated(OrderCreatedEvent {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
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
        let event = Event::OrderUpdated(OrderUpdatedEvent {
            order_id: 1,
            updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
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
        let event = Event::OrderCancelled(OrderCancelledEvent { order_id: 1 });
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
        let event = Event::OrderCreated(OrderCreatedEvent {
            order_id: 2,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        });
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
        let event = Event::OrderUpdated(OrderUpdatedEvent {
            order_id: 2,
            updated_items: vec!["Item 3".to_string(), "Item 4".to_string()],
        });
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
        let event = Event::OrderCancelled(OrderCancelledEvent { order_id: 2 });
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
