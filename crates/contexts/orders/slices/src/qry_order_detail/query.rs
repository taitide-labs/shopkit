use serde::{Deserialize, Serialize};
use shopkit_primitives::ids::{OrderId, ReservationId, UserId};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct OrderDetailItemView {
    pub sku: String,
    pub unit_price_cents: i64,
    pub quantity: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct OrderDetailResponse {
    pub order_id: OrderId,
    pub user_id: UserId,
    pub reservation_id: ReservationId,
    pub coupon_code: Option<String>,
    pub status: String,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub payable_cents: i64,
    pub items: Vec<OrderDetailItemView>,
}
