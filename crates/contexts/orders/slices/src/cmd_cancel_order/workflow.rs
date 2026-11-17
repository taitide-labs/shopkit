use shopkit_contracts::events::IntegrationEvent;
use shopkit_orders_domain::errors::OrderDomainError;
use shopkit_orders_domain::order_state::{CancelledOrder, OrderStatus};
use shopkit_orders_domain::transitions::cancel_order;
use shopkit_primitives::ids::{OrderId, ReservationId, UserId};
use time::OffsetDateTime;

use super::command::CancelOrderResponse;

pub fn prepare_cancel(
    order_id: OrderId,
    raw_status: &str,
    cancelled_at: OffsetDateTime,
) -> Result<CancelledOrder, OrderDomainError> {
    let current_status = OrderStatus::parse(raw_status)?;
    cancel_order(order_id, current_status, cancelled_at)
}

#[must_use]
pub fn order_cancelled_event(
    cancelled: CancelledOrder,
    user_id: UserId,
    reservation_id: ReservationId,
) -> IntegrationEvent {
    IntegrationEvent::OrderCancelled {
        order_id: cancelled.order_id,
        user_id,
        reservation_id,
        occurred_at: cancelled.cancelled_at,
    }
}

#[must_use]
pub fn to_response(cancelled: CancelledOrder) -> CancelOrderResponse {
    CancelOrderResponse {
        order_id: cancelled.order_id,
        status: OrderStatus::Cancelled.as_str().to_owned(),
    }
}
