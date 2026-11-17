use serde::{Deserialize, Serialize};
use shopkit_orders_domain::errors::OrderDomainError;
use shopkit_orders_domain::order_state::OrderLine;
use shopkit_primitives::errors::PrimitiveError;
use shopkit_primitives::ids::{OrderId, ReservationId, UserId};
use shopkit_primitives::simple_types::{CouponCode, Money, Quantity, Sku};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PlaceOrderItemDto {
    pub sku: String,
    pub unit_price_cents: i64,
    pub quantity: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PlaceOrderBody {
    pub order_id: Option<Uuid>,
    pub user_id: Uuid,
    pub coupon_code: Option<String>,
    pub items: Vec<PlaceOrderItemDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceOrderInput {
    pub order_id: OrderId,
    pub user_id: UserId,
    pub coupon_code: Option<CouponCode>,
    pub items: Vec<OrderLine>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParsePlaceOrderError {
    #[error(transparent)]
    Primitive(#[from] PrimitiveError),
    #[error(transparent)]
    Domain(#[from] OrderDomainError),
}

impl TryFrom<PlaceOrderBody> for PlaceOrderInput {
    type Error = ParsePlaceOrderError;

    fn try_from(body: PlaceOrderBody) -> Result<Self, Self::Error> {
        if body.items.is_empty() {
            return Err(ParsePlaceOrderError::Domain(OrderDomainError::EmptyItems));
        }
        let mut items = Vec::with_capacity(body.items.len());
        for raw in body.items {
            items.push(OrderLine {
                sku: Sku::try_new(&raw.sku)?,
                unit_price: Money::from_cents(raw.unit_price_cents)?,
                quantity: Quantity::try_new(raw.quantity)?,
            });
        }
        let coupon_code = body
            .coupon_code
            .map(|raw| CouponCode::try_new(&raw))
            .transpose()?;
        Ok(Self {
            order_id: body
                .order_id
                .map_or_else(OrderId::new_v4, OrderId::from_uuid),
            user_id: UserId::from_uuid(body.user_id),
            coupon_code,
            items,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct PlaceOrderResponse {
    pub order_id: OrderId,
    pub reservation_id: ReservationId,
    pub status: String,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub payable_cents: i64,
}
