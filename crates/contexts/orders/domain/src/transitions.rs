use shopkit_primitives::ids::{OrderId, ReservationId, UserId};
use shopkit_primitives::simple_types::{CouponCode, DiscountRule};
use time::OffsetDateTime;

use crate::errors::OrderDomainError;
use crate::order_state::{CancelledOrder, OrderLine, OrderStatus, UnpaidOrder};
use crate::pricing::calculate_pricing;

pub fn place_order(
    order_id: OrderId,
    user_id: UserId,
    reservation_id: ReservationId,
    coupon_code: Option<CouponCode>,
    items: Vec<OrderLine>,
    discount_rule: Option<DiscountRule>,
    created_at: OffsetDateTime,
) -> Result<UnpaidOrder, OrderDomainError> {
    let pricing = calculate_pricing(&items, discount_rule)?;
    Ok(UnpaidOrder {
        order_id,
        user_id,
        reservation_id,
        coupon_code,
        items,
        pricing,
        created_at,
    })
}

pub fn cancel_order(
    order_id: OrderId,
    current_status: OrderStatus,
    cancelled_at: OffsetDateTime,
) -> Result<CancelledOrder, OrderDomainError> {
    match current_status {
        OrderStatus::Unpaid => Ok(CancelledOrder {
            order_id,
            cancelled_at,
        }),
        OrderStatus::Cancelled => Err(OrderDomainError::AlreadyCancelled),
        OrderStatus::Paid => Err(OrderDomainError::InvalidStatusTransition(
            current_status.as_str().to_owned(),
        )),
    }
}
