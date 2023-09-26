#[derive(Debug)]
#[allow(dead_code)]
pub enum OrderCommand {
    Create(CreateOrderCommand),
    Update(UpdateOrderCommand),
    Cancel(CancelOrderCommand),
}

#[derive(Debug)]
pub struct CreateOrderCommand {
    pub order_id: u32,
    pub customer_name: String,
    pub items: Vec<String>,
}

#[derive(Debug)]
pub struct UpdateOrderCommand {
    pub order_id: u32,
    pub new_items: Vec<String>,
}

#[derive(Debug)]
pub struct CancelOrderCommand {
    pub order_id: u32,
}

impl OrderCommand {
    #[allow(dead_code)]
    pub fn id(&self) -> u32 {
        match self {
            OrderCommand::Create(c) => c.order_id.to_owned(),
            OrderCommand::Update(c) => c.order_id.to_owned(),
            OrderCommand::Cancel(c) => c.order_id.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum OrderEvent {
    Created(OrderCreatedEvent),
    Updated(OrderUpdatedEvent),
    Cancelled(OrderCancelledEvent),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderCreatedEvent {
    pub order_id: u32,
    pub customer_name: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderUpdatedEvent {
    pub order_id: u32,
    pub updated_items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderCancelledEvent {
    pub order_id: u32,
}

impl OrderEvent {
    #[allow(dead_code)]
    pub fn id(&self) -> u32 {
        match self {
            OrderEvent::Created(c) => c.order_id.to_owned(),
            OrderEvent::Updated(c) => c.order_id.to_owned(),
            OrderEvent::Cancelled(c) => c.order_id.to_owned(),
        }
    }
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum ShipmentCommand {
    Create(CreateShipmentCommand),
}

#[derive(Debug, PartialEq)]
pub struct CreateShipmentCommand {
    pub shipment_id: u32,
    pub order_id: u32,
    pub customer_name: String,
    pub items: Vec<String>,
}
