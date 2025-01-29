use fmodel_rust::specification::ViewTestSpecification;
use fmodel_rust::view::View;

use crate::api::{OrderCreatedEvent, OrderViewState, ShipmentCreatedEvent, ShipmentViewState};

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

fn merged_view<'a>() -> View<'a, (OrderViewState, ShipmentViewState), Event> {
    order_view().merge(self::shipment_view())
}

#[test]
fn order_created_view_test() {
    let order_created_event = Event::OrderCreated(OrderCreatedEvent {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });

    ViewTestSpecification::default()
        .for_view(self::order_view())
        .given(vec![order_created_event.clone()])
        .then(OrderViewState {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
            is_cancelled: false,
        });

    ViewTestSpecification::default()
        .for_view(merged_view())
        .given(vec![order_created_event])
        .then((
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
            },
        ));
}
#[test]

fn shipment_created_view_test() {
    let shipment_created_event = Event::ShipmentCreated(ShipmentCreatedEvent {
        shipment_id: 1,
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });

    ViewTestSpecification::default()
        .for_view(merged_view())
        .given(vec![shipment_created_event.clone()])
        .then((
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
            },
        ));
}
