use std::str::FromStr;

use uuid::Uuid;

use crate::ids::{OrderId, ReservationId, UserCouponId, UserId};

#[test]
fn uuid_round_trip_preserves_identity() {
    let raw = Uuid::new_v4();
    let user_id = UserId::from_uuid(raw);
    assert_eq!(user_id.as_uuid(), raw);

    let parsed = UserId::from_str(&user_id.to_string()).expect("valid uuid");
    assert_eq!(parsed, user_id);

    let order_id = OrderId::new_v4();
    let reservation_id = ReservationId::new_v4();
    let user_coupon_id = UserCouponId::new_v4();
    assert_ne!(order_id.as_uuid(), reservation_id.as_uuid());
    assert_ne!(reservation_id.as_uuid(), user_coupon_id.as_uuid());
}
