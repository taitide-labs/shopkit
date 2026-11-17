use shopkit_contracts::ports::{
    BoxFuture, CouponPortError, CouponRedemptionPort, RedeemCouponRequest,
};
use shopkit_primitives::ids::{OrderId, UserId};
use shopkit_primitives::simple_types::{DiscountRule, Money};
use sqlx::PgPool;
use uuid::Uuid;

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

#[derive(sqlx::FromRow)]
struct UserCouponLockRow {
    user_coupon_id: Uuid,
    status: String,
    order_id: Option<Uuid>,
    min_spend_cents: i64,
    discount_cents: i64,
}

fn evaluate_redemption(
    locked: &UserCouponLockRow,
    request: &RedeemCouponRequest,
) -> Result<(DiscountRule, bool), CouponPortError> {
    let target_order_id = request.order_id.as_uuid();
    let is_available = locked.status == "AVAILABLE" && locked.order_id.is_none();
    let is_same_order = locked.status == "REDEEMED" && locked.order_id == Some(target_order_id);

    if !is_available && !is_same_order {
        return Err(CouponPortError::CouponAlreadyUsed);
    }

    let min_spend = Money::from_cents(locked.min_spend_cents)
        .map_err(|err| CouponPortError::Unexpected(err.to_string()))?;
    let discount = Money::from_cents(locked.discount_cents)
        .map_err(|err| CouponPortError::Unexpected(err.to_string()))?;
    let rule = DiscountRule::try_new(min_spend, discount)
        .map_err(|err| CouponPortError::Unexpected(err.to_string()))?;

    if request.subtotal < rule.min_spend {
        return Err(CouponPortError::MinimumSpendNotMet {
            min_spend_cents: rule.min_spend.as_cents(),
        });
    }

    Ok((rule, is_available))
}

async fn redeem_coupon_inner(
    pool: &PgPool,
    request: &RedeemCouponRequest,
) -> Result<DiscountRule, CouponPortError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|err| CouponPortError::Unexpected(err.to_string()))?;

    let row = sqlx::query_as::<_, UserCouponLockRow>(
        "SELECT uc.user_coupon_id, uc.status, uc.order_id, c.min_spend_cents, c.discount_cents
         FROM user_coupons AS uc
         INNER JOIN coupons AS c ON c.code = uc.code
         WHERE uc.user_id = $1 AND uc.code = $2
         FOR UPDATE OF uc",
    )
    .bind(request.user_id.as_uuid())
    .bind(request.code.as_str())
    .fetch_optional(&mut *tx)
    .await
    .map_err(|err| CouponPortError::Unexpected(err.to_string()))?;

    let Some(locked) = row else {
        return Err(CouponPortError::CouponNotOwned);
    };

    let (rule, needs_update) = evaluate_redemption(&locked, request)?;

    if needs_update {
        sqlx::query(
            "UPDATE user_coupons
             SET status = 'REDEEMED', order_id = $2, redeemed_at = NOW()
             WHERE user_coupon_id = $1",
        )
        .bind(locked.user_coupon_id)
        .bind(request.order_id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(|err| CouponPortError::Unexpected(err.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|err| CouponPortError::Unexpected(err.to_string()))?;

    Ok(rule)
}

async fn release_coupon_inner(
    pool: &PgPool,
    user_id: UserId,
    order_id: OrderId,
) -> Result<(), CouponPortError> {
    sqlx::query(
        "UPDATE user_coupons
         SET status = 'AVAILABLE', order_id = NULL, redeemed_at = NULL
         WHERE user_id = $1 AND order_id = $2 AND status = 'REDEEMED'",
    )
    .bind(user_id.as_uuid())
    .bind(order_id.as_uuid())
    .execute(pool)
    .await
    .map_err(|err| CouponPortError::Unexpected(err.to_string()))?;

    Ok(())
}

impl CouponRedemptionPort for PgCouponRedemptionPort {
    fn redeem_coupon<'a>(
        &'a self,
        request: &'a RedeemCouponRequest,
    ) -> BoxFuture<'a, Result<DiscountRule, CouponPortError>> {
        Box::pin(async move { redeem_coupon_inner(&self.pool, request).await })
    }

    fn release_coupon(
        &self,
        user_id: UserId,
        order_id: OrderId,
    ) -> BoxFuture<'_, Result<(), CouponPortError>> {
        Box::pin(async move { release_coupon_inner(&self.pool, user_id, order_id).await })
    }
}

#[cfg(test)]
mod tests;
