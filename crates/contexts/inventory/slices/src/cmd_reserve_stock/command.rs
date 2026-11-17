use serde::{Deserialize, Serialize};
use shopkit_contracts::ports::{ReserveStockItem, ReserveStockRequest};
use shopkit_primitives::errors::PrimitiveError;
use shopkit_primitives::ids::{OrderId, ReservationId};
use shopkit_primitives::simple_types::{Quantity, Sku};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ReserveStockLineDto {
    pub sku: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ReserveStockBody {
    pub order_id: Uuid,
    pub items: Vec<ReserveStockLineDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ReserveStockResponse {
    pub reservation_id: ReservationId,
    pub order_id: OrderId,
}

impl TryFrom<ReserveStockBody> for ReserveStockRequest {
    type Error = PrimitiveError;

    fn try_from(body: ReserveStockBody) -> Result<Self, Self::Error> {
        let mut items = Vec::with_capacity(body.items.len());
        for line in body.items {
            items.push(ReserveStockItem {
                sku: Sku::try_new(&line.sku)?,
                quantity: Quantity::try_new(line.quantity)?,
            });
        }
        Ok(Self {
            order_id: OrderId::from_uuid(body.order_id),
            items,
        })
    }
}
