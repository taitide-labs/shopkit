use shopkit_primitives::ids::{OrderId, ReservationId, UserId};
use shopkit_primitives::simple_types::{Money, Quantity, Sku};
use time::macros::datetime;

use crate::errors::OrderDomainError;
use crate::order_state::{OrderLine, OrderStatus};
use crate::transitions::{cancel_order, place_order};

#[test]
fn order_state_machine_transitions_from_unpaid_to_cancelled() {
    let now = datetime!(2026-11-18 0:00:00 UTC);
    let order = place_order(
        OrderId::new_v4(),
        UserId::new_v4(),
        ReservationId::new_v4(),
        None,
        vec![OrderLine {
            sku: Sku::try_new("SKU-1").expect("valid sku"),
            unit_price: Money::from_cents(2_000).expect("valid"),
            quantity: Quantity::try_new(1).expect("valid"),
        }],
        None,
        now,
    )
    .expect("placed");

    let cancelled = cancel_order(order.order_id, OrderStatus::Unpaid, now).expect("cancelled");
    assert_eq!(cancelled.order_id, order.order_id);

    assert_eq!(
        cancel_order(order.order_id, OrderStatus::Cancelled, now),
        Err(OrderDomainError::AlreadyCancelled)
    );
    assert!(matches!(
        cancel_order(order.order_id, OrderStatus::Paid, now),
        Err(OrderDomainError::InvalidStatusTransition(_))
    ));
}
