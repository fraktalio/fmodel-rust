use fmodel_rust::decider::Decider;
use fmodel_rust::specification::DeciderTestSpecification;

use crate::api::{
    CreateOrderCommand, CreateShipmentCommand, OrderCancelledEvent, OrderCommand,
    OrderCreatedEvent, OrderEvent, OrderState, OrderUpdatedEvent, ShipmentCommand,
    ShipmentCreatedEvent, ShipmentEvent, ShipmentState,
};
use crate::application::Event::{OrderCreated, ShipmentCreated};
use crate::application::{command_from_sum, event_from_sum, sum_to_event, Command, Event};

mod api;
mod application;

fn order_decider<'a>() -> Decider<'a, OrderCommand, OrderState, OrderEvent> {
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
                ShipmentEvent::Created(evt) => {
                    new_state.shipment_id = evt.shipment_id;
                    new_state.order_id = evt.order_id;
                    new_state.customer_name = evt.customer_name.to_owned();
                    new_state.items = evt.items.to_owned();
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

fn combined_decider<'a>() -> Decider<'a, Command, (OrderState, ShipmentState), Event> {
    order_decider()
        .combine(shipment_decider())
        .map_command(&command_from_sum) // Decider<Command, (OrderState, ShipmentState), Sum<OrderEvent, ShipmentEvent>>
        .map_event(&event_from_sum, &sum_to_event)
}

#[test]
fn create_order_event_sourced_test() {
    let create_order_command = CreateOrderCommand {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    };

    // Test the OrderDecider (event-sourced)
    DeciderTestSpecification::default()
        .for_decider(self::order_decider()) // Set the decider
        .given(vec![]) // no existing events
        .when(OrderCommand::Create(create_order_command.clone())) // Create an Order
        .then(vec![OrderEvent::Created(OrderCreatedEvent {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]);

    // Test the Decider that combines OrderDecider and ShipmentDecider and can handle both OrderCommand and ShipmentCommand and produce Event (event-sourced)
    DeciderTestSpecification::default()
        .for_decider(self::combined_decider())
        .given(vec![])
        .when(Command::OrderCreate(create_order_command.clone()))
        .then(vec![OrderCreated(OrderCreatedEvent {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]);
}

#[test]
fn create_shipment_event_sourced_test() {
    let create_shipment_command = CreateShipmentCommand {
        shipment_id: 1,
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    };

    DeciderTestSpecification::default()
        .for_decider(self::combined_decider())
        .given(vec![])
        .when(Command::ShipmentCreate(create_shipment_command.clone()))
        .then(vec![ShipmentCreated(ShipmentCreatedEvent {
            shipment_id: 1,
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]);
}

#[test]
fn create_order_state_stored_test() {
    let create_order_command = CreateOrderCommand {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    };

    // Test the OrderDecider (state stored)
    DeciderTestSpecification::default()
        .for_decider(self::order_decider()) // Set the decider
        .given_state(None) // no existing state
        .when(OrderCommand::Create(create_order_command.clone())) // Create an Order
        .then_state(OrderState {
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
            is_cancelled: false,
        });
}

#[test]
fn create_shipment_state_stored_test() {
    let create_shipment_command = CreateShipmentCommand {
        shipment_id: 1,
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    };
    // Test the Decider (state stored) that combines OrderDecider and ShipmentDecider and can handle both OrderCommand and ShipmentCommand and produce a tuple of (OrderState, ShipmentState)
    DeciderTestSpecification::default()
        .for_decider(self::combined_decider())
        .given_state(None)
        .when(Command::ShipmentCreate(create_shipment_command.clone()))
        .then_state((
            OrderState {
                order_id: 0,
                customer_name: "".to_string(),
                items: Vec::new(),
                is_cancelled: false,
            },
            ShipmentState {
                shipment_id: 1,
                order_id: 1,
                customer_name: "John Doe".to_string(),
                items: vec!["Item 1".to_string(), "Item 2".to_string()],
            },
        ));
}
