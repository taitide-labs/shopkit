use shopkit_primitives::errors::PrimitiveError;
use shopkit_primitives::ids::UserId;
use uuid::Uuid;

use super::super::command::{ClaimCouponBody, ClaimCouponInput};

#[test]
fn try_new_normalizes_coupon_code_and_preserves_user_id() {
    let user_uuid = Uuid::new_v4();
    let input = ClaimCouponInput::try_new(" save-20 ", ClaimCouponBody { user_id: user_uuid })
        .expect("valid claim input");

    assert_eq!(input.user_id, UserId::from_uuid(user_uuid));
    assert_eq!(input.code.as_str(), "SAVE-20");
}

#[test]
fn try_new_rejects_invalid_or_overlong_coupon_codes() {
    let body = ClaimCouponBody {
        user_id: Uuid::new_v4(),
    };

    assert_eq!(
        ClaimCouponInput::try_new("   ", body),
        Err(PrimitiveError::InvalidCouponCode("   ".to_owned()))
    );
    assert_eq!(
        ClaimCouponInput::try_new("SAVE_20", body),
        Err(PrimitiveError::InvalidCouponCode("SAVE_20".to_owned()))
    );
    let too_long = "A".repeat(33);
    assert_eq!(
        ClaimCouponInput::try_new(&too_long, body),
        Err(PrimitiveError::InvalidCouponCode(too_long))
    );
    let boundary = "A".repeat(32);
    assert_eq!(
        ClaimCouponInput::try_new(&boundary, body)
            .expect("32 chars accepted")
            .code
            .as_str(),
        boundary
    );
}
