use async_trait::async_trait;
use derive_more::Display;
use fmodel_rust::saga::Saga;
use fmodel_rust::saga_manager::{ActionPublisher, SagaManager};
use std::error::Error;

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

/// Error type for the saga manager
#[derive(Debug, Display)]
#[allow(dead_code)]
enum SagaManagerError {
    PublishAction(String),
}

impl Error for SagaManagerError {}

/// Simple action publisher that just returns the action/command.
/// It is used for testing. In real life, it would publish the action/command to some external system. or to an aggregate that is able to handel the action/command.
struct SimpleActionPublisher;

impl SimpleActionPublisher {
    fn new() -> Self {
        SimpleActionPublisher {}
    }
}

#[async_trait]
impl ActionPublisher<ShipmentCommand, SagaManagerError> for SimpleActionPublisher {
    async fn publish(
        &self,
        action: &[ShipmentCommand],
    ) -> Result<Vec<ShipmentCommand>, SagaManagerError> {
        Ok(Vec::from(action))
    }
}

#[tokio::test]
async fn test() {
    let saga: Saga<OrderEvent, ShipmentCommand> = saga();
    let order_created_event = OrderEvent::Created(OrderCreatedEvent {
        order_id: 1,
        customer_name: "John Doe".to_string(),
        items: vec!["Item 1".to_string(), "Item 2".to_string()],
    });

    let saga_manager = SagaManager::new(SimpleActionPublisher::new(), saga);
    let result = saga_manager.handle(&order_created_event).await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        vec![ShipmentCommand::Create(CreateShipmentCommand {
            shipment_id: 1,
            order_id: 1,
            customer_name: "John Doe".to_string(),
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
        })]
    );
}
