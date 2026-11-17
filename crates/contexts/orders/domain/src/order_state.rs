use serde::{Deserialize, Serialize};
use shopkit_primitives::ids::{OrderId, ReservationId, UserId};
use shopkit_primitives::simple_types::{CouponCode, Money, Quantity, Sku};
use time::OffsetDateTime;

use crate::errors::OrderDomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    Unpaid,
    Paid,
    Cancelled,
}

impl OrderStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unpaid => "UNPAID",
            Self::Paid => "PAID",
            Self::Cancelled => "CANCELLED",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, OrderDomainError> {
        match raw {
            "UNPAID" => Ok(Self::Unpaid),
            "PAID" => Ok(Self::Paid),
            "CANCELLED" => Ok(Self::Cancelled),
            other => Err(OrderDomainError::InvalidStatusTransition(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderLine {
    pub sku: Sku,
    pub unit_price: Money,
    pub quantity: Quantity,
}

impl OrderLine {
    pub fn line_total(&self) -> Result<Money, OrderDomainError> {
        self.unit_price
            .multiply_quantity(self.quantity)
            .map_err(|_err| OrderDomainError::PricingOverflow)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderPricing {
    pub subtotal: Money,
    pub discount: Money,
    pub payable: Money,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnpaidOrder {
    pub order_id: OrderId,
    pub user_id: UserId,
    pub reservation_id: ReservationId,
    pub coupon_code: Option<CouponCode>,
    pub items: Vec<OrderLine>,
    pub pricing: OrderPricing,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancelledOrder {
    pub order_id: OrderId,
    pub cancelled_at: OffsetDateTime,
}
