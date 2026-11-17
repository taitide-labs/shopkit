use shopkit_contracts::events::IntegrationEvent;
use shopkit_primitives::ids::{OrderId, ReservationId, UserId};
use shopkit_primitives::simple_types::Money;
use time::macros::datetime;

use crate::event_bus::EventBus;

#[test]
fn event_bus_publishes_to_active_subscribers() {
    let bus = EventBus::new(16);
    let mut rx = bus.subscribe();
    let occurred_at = datetime!(2026-11-18 0:00:00 UTC);
    let event = IntegrationEvent::OrderPlaced {
        order_id: OrderId::new_v4(),
        user_id: UserId::new_v4(),
        reservation_id: ReservationId::new_v4(),
        payable_cents: Money::from_cents(5000).expect("valid money"),
        occurred_at,
    };

    bus.publish(event.clone());
    let received = rx.try_recv().expect("received published event");
    assert_eq!(received, event);
    assert_eq!(received.event_type(), "OrderPlaced");
}
