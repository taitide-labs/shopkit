use serde::{Deserialize, Serialize};
use shopkit_primitives::ids::{OrderId, UserCouponId, UserId};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UserCouponItemView {
    pub user_coupon_id: UserCouponId,
    pub code: String,
    pub status: String,
    pub min_spend_cents: i64,
    pub discount_cents: i64,
    pub order_id: Option<OrderId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UserCouponsResponse {
    pub user_id: UserId,
    pub coupons: Vec<UserCouponItemView>,
}
