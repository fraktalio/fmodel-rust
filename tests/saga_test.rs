use fmodel_rust::saga::{ActionComputation, Saga};

use crate::api::{
    CreateShipmentCommand, OrderCommand, OrderCreatedEvent, OrderEvent, ShipmentCommand,
    ShipmentEvent, UpdateOrderCommand,
};
use crate::application::{event_from_sum, sum_to_command, Command, Event};

mod api;
mod application;

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
                vec![OrderCommand::Update(UpdateOrderCommand {
                    order_id: evt.order_id,
                    new_items: evt.items.to_owned(),
                })]
            }
        }),
    }
}

#[test]
fn test() {
    let order_saga: Saga<OrderEvent, ShipmentCommand> = order_saga();
    let order_saga2: Saga<OrderEvent, ShipmentCommand> = crate::order_saga();
    let shipment_saga: Saga<ShipmentEvent, OrderCommand> = shipment_saga();
    let combined_saga = order_saga2
        .combine(shipment_saga)
        .map_action(&sum_to_command)
        .map_action_result(&event_from_sum);
    let order_created_event = OrderEvent::Created(OrderCreatedEvent {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });
    let commands = order_saga.compute_new_actions(&order_created_event);
    assert_eq!(
        commands,
        [ShipmentCommand::Create(CreateShipmentCommand {
            shipment_id: 1,
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]
    );
    let order_created_event2 = Event::OrderCreated(OrderCreatedEvent {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });
    let combined_commands = combined_saga.compute_new_actions(&order_created_event2);
    assert_eq!(
        combined_commands,
        [Command::ShipmentCreate(CreateShipmentCommand {
            shipment_id: 1,
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]
    );
}
