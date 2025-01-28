use fmodel_rust::view::{View, ViewStateComputation};

use crate::api::{
    OrderCancelledEvent, OrderCreatedEvent, OrderUpdatedEvent, OrderViewState,
    ShipmentCreatedEvent, ShipmentViewState,
};

use crate::application::Event;

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

#[test]
fn test() {
    let order_view: View<OrderViewState, Event> = order_view();
    let order_view2: View<OrderViewState, Event> = crate::order_view();
    let shipment_view: View<ShipmentViewState, Event> = shipment_view();

    let merged_view = order_view2.merge(shipment_view);

    let order_created_event = Event::OrderCreated(OrderCreatedEvent {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });

    let new_state = order_view.compute_new_state(None, &[&order_created_event]);
    assert_eq!(
        new_state,
        OrderViewState {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
            is_cancelled: false,
        }
    );
    let order_created_event2 = Event::OrderCreated(OrderCreatedEvent {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });

    let new_merged_state2 = merged_view.compute_new_state(None, &[&order_created_event2]);
    assert_eq!(
        new_merged_state2,
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

    let shipment_created_event2 = Event::ShipmentCreated(ShipmentCreatedEvent {
        shipment_id: 1,
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });
    let new_merged_state3 = merged_view.compute_new_state(None, &[&shipment_created_event2]);
    assert_eq!(
        new_merged_state3,
        (
            OrderViewState {
                order_id: 0,
                customer_name: "".to_string(),
                items: Vec::new(),
                is_cancelled: false,
            },
            ShipmentViewState {
                shipment_id: 1,
                order_id: 1,
                customer_name: "John Doe".to_string(),
                items: vec!["Item 1".to_string(), "Item 2".to_string()],
            }
        )
    );

    let order_updated_event = Event::OrderUpdated(OrderUpdatedEvent {
        order_id: 1,
        updated_items: vec![
            "Item 11".to_string(),
            "Item 22".to_string(),
            "Item 33".to_string(),
        ],
    });
    let new_state = order_view.compute_new_state(Some(new_state), &[&order_updated_event]);
    assert_eq!(
        new_state,
        OrderViewState {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec![
                "Item 11".to_string(),
                "Item 22".to_string(),
                "Item 33".to_string()
            ],
            is_cancelled: false,
        }
    );

    let order_canceled_event = Event::OrderCancelled(OrderCancelledEvent { order_id: 1 });
    let new_state = order_view.compute_new_state(Some(new_state), &[&order_canceled_event]);
    assert_eq!(
        new_state,
        OrderViewState {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec![
                "Item 11".to_string(),
                "Item 22".to_string(),
                "Item 33".to_string()
            ],
            is_cancelled: true,
        }
    );
}
