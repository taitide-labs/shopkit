use shopkit_contracts::ports::{
    BoxFuture, CouponPortError, CouponRedemptionPort, RedeemCouponRequest,
};
use shopkit_primitives::ids::{OrderId, UserId};
use shopkit_primitives::simple_types::DiscountRule;
use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct PgCouponRedemptionPort {
    pool: PgPool,
}

impl PgCouponRedemptionPort {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[must_use]
    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl CouponRedemptionPort for PgCouponRedemptionPort {
    fn redeem_coupon<'a>(
        &'a self,
        _request: &'a RedeemCouponRequest,
    ) -> BoxFuture<'a, Result<DiscountRule, CouponPortError>> {
        Box::pin(async move { Err(CouponPortError::Unexpected("planned slice".to_owned())) })
    }

    fn release_coupon(
        &self,
        _user_id: UserId,
        _order_id: OrderId,
    ) -> BoxFuture<'_, Result<(), CouponPortError>> {
        Box::pin(async move { Err(CouponPortError::Unexpected("planned slice".to_owned())) })
    }
}
