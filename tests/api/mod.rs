// ###################################################################
// ############################ Order API ############################
// ###################################################################

/// The state of the Order entity
#[derive(Debug, Clone, PartialEq)]
pub struct OrderState {
    pub order_id: u32,
    pub customer_name: String,
    pub items: Vec<String>,
    pub is_cancelled: bool,
}

/// The state of the ViewOrder entity / It represents the Query Model
#[derive(Debug, Clone, PartialEq)]
pub struct OrderViewState {
    pub order_id: u32,
    pub customer_name: String,
    pub items: Vec<String>,
    pub is_cancelled: bool,
}

/// All variants of Order commands
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum OrderCommand {
    Create(CreateOrderCommand),
    Update(UpdateOrderCommand),
    Cancel(CancelOrderCommand),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateOrderCommand {
    pub order_id: u32,
    pub customer_name: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateOrderCommand {
    pub order_id: u32,
    pub new_items: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CancelOrderCommand {
    pub order_id: u32,
}

/// Provides a way to get the id of the Order commands
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

/// All variants of Order events
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

/// Provides a way to get the id of the Order events
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

// ######################################################################
// ############################ Shipment API ############################
// ######################################################################

/// The state of the Shipment entity
#[derive(Debug, Clone, PartialEq)]
pub struct ShipmentState {
    pub shipment_id: u32,
    pub order_id: u32,
    pub customer_name: String,
    pub items: Vec<String>,
}

/// The state of the ViewShipment entity / It represents the Query Model
#[derive(Debug, Clone, PartialEq)]
pub struct ShipmentViewState {
    pub shipment_id: u32,
    pub order_id: u32,
    pub customer_name: String,
    pub items: Vec<String>,
}

/// All variants of Shipment commands
#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum ShipmentCommand {
    Create(CreateShipmentCommand),
}

#[derive(Debug, PartialEq, Clone)]
pub struct CreateShipmentCommand {
    pub shipment_id: u32,
    pub order_id: u32,
    pub customer_name: String,
    pub items: Vec<String>,
}

/// Provides a way to get the id of the Shipment commands
impl ShipmentCommand {
    #[allow(dead_code)]
    pub fn id(&self) -> u32 {
        match self {
            ShipmentCommand::Create(c) => c.shipment_id.to_owned(),
        }
    }
}

/// All variants of Shipment events
#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum ShipmentEvent {
    Created(ShipmentCreatedEvent),
}
#[derive(Debug, PartialEq, Clone)]
pub struct ShipmentCreatedEvent {
    pub shipment_id: u32,
    pub order_id: u32,
    pub customer_name: String,
    pub items: Vec<String>,
}

/// Provides a way to get the id of the Shipment events
impl ShipmentEvent {
    #[allow(dead_code)]
    pub fn id(&self) -> u32 {
        match self {
            ShipmentEvent::Created(c) => c.shipment_id.to_owned(),
        }
    }
}
