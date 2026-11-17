use shopkit_primitives::ids::{OrderId, UserCouponId, UserId};
use sqlx::PgPool;
use uuid::Uuid;

use super::query::{UserCouponItemView, UserCouponsResponse};

#[derive(sqlx::FromRow)]
struct UserCouponRow {
    user_coupon_id: Uuid,
    code: String,
    status: String,
    min_spend_cents: i64,
    discount_cents: i64,
    order_id: Option<Uuid>,
}

pub async fn handle(pool: &PgPool, user_id: UserId) -> Result<UserCouponsResponse, sqlx::Error> {
    let rows = sqlx::query_as::<_, UserCouponRow>(
        "SELECT uc.user_coupon_id, uc.code, uc.status, c.min_spend_cents, c.discount_cents, uc.order_id
         FROM user_coupons AS uc
         INNER JOIN coupons AS c ON c.code = uc.code
         WHERE uc.user_id = $1
         ORDER BY uc.claimed_at DESC, uc.user_coupon_id ASC",
    )
    .bind(user_id.as_uuid())
    .fetch_all(pool)
    .await?;

    let coupons = rows
        .into_iter()
        .map(|row| UserCouponItemView {
            user_coupon_id: UserCouponId::from_uuid(row.user_coupon_id),
            code: row.code,
            status: row.status,
            min_spend_cents: row.min_spend_cents,
            discount_cents: row.discount_cents,
            order_id: row.order_id.map(OrderId::from_uuid),
        })
        .collect();

    Ok(UserCouponsResponse { user_id, coupons })
}
