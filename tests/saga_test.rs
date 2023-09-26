use fmodel_rust::saga::Saga;

use crate::api::{CreateShipmentCommand, OrderCreatedEvent, OrderEvent, ShipmentCommand};

mod api;

fn saga<'a>() -> Saga<'a, OrderEvent, ShipmentCommand> {
    Saga {
        react: Box::new(|event| match event {
            OrderEvent::Created(created_event) => {
                vec![ShipmentCommand::Create(CreateShipmentCommand {
                    shipment_id: created_event.order_id,
                    order_id: created_event.order_id,
                    customer_name: created_event.customer_name.to_owned(),
                    items: created_event.items.to_owned(),
                })]
            }
            OrderEvent::Updated(_updated_event) => {
                vec![]
            }
            OrderEvent::Cancelled(_cancelled_event) => {
                vec![]
            }
        }),
    }
}

#[test]
fn test() {
    let saga: Saga<OrderEvent, ShipmentCommand> = saga();
    let order_created_event = OrderEvent::Created(OrderCreatedEvent {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });
    let commands = (saga.react)(&order_created_event);
    assert_eq!(
        commands,
        vec![ShipmentCommand::Create(CreateShipmentCommand {
            shipment_id: 1,
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]
    );
}
