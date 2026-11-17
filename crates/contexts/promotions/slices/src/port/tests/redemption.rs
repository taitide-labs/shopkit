use shopkit_contracts::ports::{CouponPortError, CouponRedemptionPort, RedeemCouponRequest};
use shopkit_primitives::ids::{OrderId, UserId};
use shopkit_primitives::simple_types::{CouponCode, DiscountRule, Money};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::super::PgCouponRedemptionPort;

async fn seed_user_coupon(pool: &PgPool, user_id: UserId, code: &str, min: i64, discount: i64) {
    sqlx::query(
        "INSERT INTO coupons (code, min_spend_cents, discount_cents, total_quota, claimed_count, created_at)
         VALUES ($1, $2, $3, 10, 1, NOW())",
    )
    .bind(code)
    .bind(min)
    .bind(discount)
    .execute(pool)
    .await
    .expect("seed coupon");

    sqlx::query(
        "INSERT INTO user_coupons (user_coupon_id, user_id, code, status, claimed_at)
         VALUES ($1, $2, $3, 'AVAILABLE', NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(user_id.as_uuid())
    .bind(code)
    .execute(pool)
    .await
    .expect("seed user coupon");
}

#[sqlx::test(migrations = "../../../../migrations")]
async fn redeems_enforces_ownership_threshold_uniqueness_and_releases(pool: PgPool) {
    let port = PgCouponRedemptionPort::new(pool.clone());
    let owner = UserId::new_v4();
    let stranger = UserId::new_v4();
    let first_order = OrderId::new_v4();
    let second_order = OrderId::new_v4();
    let code = CouponCode::try_new("SAVE20").expect("valid code");
    seed_user_coupon(&pool, owner, "SAVE20", 10_000, 2_000).await;

    let not_owned = RedeemCouponRequest {
        user_id: stranger,
        order_id: first_order,
        code: code.clone(),
        subtotal: Money::from_cents(15_000).expect("valid money"),
    };
    assert_eq!(
        port.redeem_coupon(&not_owned).await,
        Err(CouponPortError::CouponNotOwned)
    );

    let below_min = RedeemCouponRequest {
        user_id: owner,
        order_id: first_order,
        code: code.clone(),
        subtotal: Money::from_cents(9_999).expect("valid money"),
    };
    assert_eq!(
        port.redeem_coupon(&below_min).await,
        Err(CouponPortError::MinimumSpendNotMet {
            min_spend_cents: 10_000
        })
    );

    let valid = RedeemCouponRequest {
        user_id: owner,
        order_id: first_order,
        code: code.clone(),
        subtotal: Money::from_cents(10_000).expect("valid money"),
    };
    let expected_rule = DiscountRule::try_new(
        Money::from_cents(10_000).expect("min"),
        Money::from_cents(2_000).expect("disc"),
    )
    .expect("rule");
    assert_eq!(port.redeem_coupon(&valid).await, Ok(expected_rule));
    assert_eq!(port.redeem_coupon(&valid).await, Ok(expected_rule));

    let other_order_req = RedeemCouponRequest {
        user_id: owner,
        order_id: second_order,
        code: code.clone(),
        subtotal: Money::from_cents(12_000).expect("valid money"),
    };
    assert_eq!(
        port.redeem_coupon(&other_order_req).await,
        Err(CouponPortError::CouponAlreadyUsed)
    );

    let row = sqlx::query(
        "SELECT status, order_id, redeemed_at IS NOT NULL AS has_redeemed_at
         FROM user_coupons WHERE user_id = $1 AND code = $2",
    )
    .bind(owner.as_uuid())
    .bind("SAVE20")
    .fetch_one(&pool)
    .await
    .expect("fetch redeemed row");
    assert_eq!(row.get::<String, _>("status"), "REDEEMED");
    assert_eq!(
        row.get::<Option<Uuid>, _>("order_id"),
        Some(first_order.as_uuid())
    );
    assert!(row.get::<bool, _>("has_redeemed_at"));

    port.release_coupon(owner, first_order)
        .await
        .expect("release");
    port.release_coupon(owner, first_order)
        .await
        .expect("idempotent release");

    let released_row = sqlx::query(
        "SELECT status, order_id, redeemed_at
         FROM user_coupons WHERE user_id = $1 AND code = $2",
    )
    .bind(owner.as_uuid())
    .bind("SAVE20")
    .fetch_one(&pool)
    .await
    .expect("fetch released row");
    assert_eq!(released_row.get::<String, _>("status"), "AVAILABLE");
    assert_eq!(released_row.get::<Option<Uuid>, _>("order_id"), None);
    assert_eq!(
        released_row.get::<Option<time::OffsetDateTime>, _>("redeemed_at"),
        None
    );

    assert_eq!(
        port.redeem_coupon(&other_order_req).await,
        Ok(expected_rule)
    );
}
