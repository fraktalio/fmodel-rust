use fmodel_rust::saga::Saga;
use fmodel_rust::saga_manager::{ActionPublisher, SagaManager};

use crate::api::{
    CreateShipmentCommand, OrderCommand, OrderCreatedEvent, OrderEvent, ShipmentCommand,
    ShipmentEvent, UpdateOrderCommand,
};
use crate::application::{event_from_sum2, sum_to_command2, Command, Event, SagaManagerError};

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

/// Simple action publisher that just returns the action/command.
/// It is used for testing. In real life, it would publish the action/command to some external system. or to an aggregate that is able to handel the action/command.
struct SimpleActionPublisher;

impl SimpleActionPublisher {
    fn new() -> Self {
        SimpleActionPublisher {}
    }
}

impl ActionPublisher<Command, SagaManagerError> for SimpleActionPublisher {
    async fn publish(&self, action: &[Command]) -> Result<Vec<Command>, SagaManagerError> {
        Ok(Vec::from(action))
    }
}

#[tokio::test]
async fn test() {
    let order_created_event = Event::OrderCreated(OrderCreatedEvent {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });

    let saga_manager = SagaManager::new(
        SimpleActionPublisher::new(),
        shipment_saga()
            .combine(order_saga())
            .map_action(&sum_to_command2)
            .map_action_result(&event_from_sum2),
    );
    let result = saga_manager.handle(&order_created_event).await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![Command::ShipmentCreate(CreateShipmentCommand {
            shipment_id: 1,
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]
    );
}
