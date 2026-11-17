use shopkit_contracts::events::IntegrationEvent;
use shopkit_orders_domain::errors::OrderDomainError;
use shopkit_primitives::ids::{OrderId, ReservationId, UserId};
use time::macros::datetime;

use super::super::workflow::{order_cancelled_event, prepare_cancel, to_response};

#[test]
fn prepare_cancel_transitions_unpaid_and_rejects_other_states() {
    let order_id = OrderId::new_v4();
    let user_id = UserId::new_v4();
    let reservation_id = ReservationId::new_v4();
    let cancelled_at = datetime!(2026-11-18 0:00:00 UTC);

    let cancelled = prepare_cancel(order_id, "UNPAID", cancelled_at).expect("valid transition");
    assert_eq!(cancelled.order_id, order_id);
    assert_eq!(cancelled.cancelled_at, cancelled_at);

    let response = to_response(cancelled);
    assert_eq!(response.order_id, order_id);
    assert_eq!(response.status, "CANCELLED");

    let event = order_cancelled_event(cancelled, user_id, reservation_id);
    assert_eq!(
        event,
        IntegrationEvent::OrderCancelled {
            order_id,
            user_id,
            reservation_id,
            occurred_at: cancelled_at,
        }
    );

    assert_eq!(
        prepare_cancel(order_id, "CANCELLED", cancelled_at),
        Err(OrderDomainError::AlreadyCancelled)
    );
    assert_eq!(
        prepare_cancel(order_id, "PAID", cancelled_at),
        Err(OrderDomainError::InvalidStatusTransition("PAID".to_owned()))
    );
    assert_eq!(
        prepare_cancel(order_id, "UNKNOWN", cancelled_at),
        Err(OrderDomainError::InvalidStatusTransition(
            "UNKNOWN".to_owned()
        ))
    );
}
