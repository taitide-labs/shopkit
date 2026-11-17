use serde::{Deserialize, Serialize};
use shopkit_primitives::errors::PrimitiveError;
use shopkit_primitives::ids::{UserCouponId, UserId};
use shopkit_primitives::simple_types::CouponCode;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ClaimCouponBody {
    pub user_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimCouponInput {
    pub user_id: UserId,
    pub code: CouponCode,
}

impl ClaimCouponInput {
    pub fn try_new(code_raw: &str, body: ClaimCouponBody) -> Result<Self, PrimitiveError> {
        Ok(Self {
            user_id: UserId::from_uuid(body.user_id),
            code: CouponCode::try_new(code_raw)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ClaimCouponResponse {
    pub user_coupon_id: UserCouponId,
    pub user_id: UserId,
    pub code: CouponCode,
    pub min_spend_cents: i64,
    pub discount_cents: i64,
}
