use fmodel_rust::view::{View, ViewStateComputation};

use crate::api::{
    OrderCancelledEvent, OrderCreatedEvent, OrderEvent, OrderUpdatedEvent, OrderViewState,
    ShipmentCreatedEvent, ShipmentEvent, ShipmentViewState,
};

use crate::application::{event_from_sum, Event};

mod api;
mod application;

fn order_view<'a>() -> View<'a, OrderViewState, OrderEvent> {
    View {
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
                ShipmentEvent::Created(evt) => {
                    new_state.shipment_id = evt.shipment_id;
                    new_state.order_id = evt.order_id;
                    new_state.customer_name = evt.customer_name.to_owned();
                    new_state.items = evt.items.to_owned();
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

#[test]
fn test() {
    let order_view: View<OrderViewState, OrderEvent> = order_view();
    let order_view2: View<OrderViewState, OrderEvent> = crate::order_view();
    let shipment_view: View<ShipmentViewState, ShipmentEvent> = shipment_view();
    let combined_view = order_view2
        .combine(shipment_view)
        .map_event(&event_from_sum);

    let order_created_event = OrderEvent::Created(OrderCreatedEvent {
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
    let new_combined_state2 = combined_view.compute_new_state(None, &[&order_created_event2]);
    assert_eq!(
        new_combined_state2,
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
    let new_combined_state3 = combined_view.compute_new_state(None, &[&shipment_created_event2]);
    assert_eq!(
        new_combined_state3,
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

    let order_updated_event = OrderEvent::Updated(OrderUpdatedEvent {
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

    let order_canceled_event = OrderEvent::Cancelled(OrderCancelledEvent { order_id: 1 });
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
