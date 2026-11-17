use serde::{Deserialize, Serialize};
use shopkit_primitives::ids::{OrderId, ReservationId, UserId};
use shopkit_primitives::simple_types::{CouponCode, Money};
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[non_exhaustive]
#[serde(tag = "type", content = "data")]
pub enum IntegrationEvent {
    OrderPlaced {
        order_id: OrderId,
        user_id: UserId,
        reservation_id: ReservationId,
        payable_cents: Money,
        occurred_at: OffsetDateTime,
    },
    OrderCancelled {
        order_id: OrderId,
        user_id: UserId,
        reservation_id: ReservationId,
        occurred_at: OffsetDateTime,
    },
    StockReserved {
        reservation_id: ReservationId,
        order_id: OrderId,
        occurred_at: OffsetDateTime,
    },
    StockReleased {
        reservation_id: ReservationId,
        order_id: OrderId,
        occurred_at: OffsetDateTime,
    },
    CouponClaimed {
        user_id: UserId,
        code: CouponCode,
        occurred_at: OffsetDateTime,
    },
}

impl IntegrationEvent {
    #[must_use]
    pub const fn event_type(&self) -> &'static str {
        match self {
            Self::OrderPlaced { .. } => "OrderPlaced",
            Self::OrderCancelled { .. } => "OrderCancelled",
            Self::StockReserved { .. } => "StockReserved",
            Self::StockReleased { .. } => "StockReleased",
            Self::CouponClaimed { .. } => "CouponClaimed",
        }
    }
}
