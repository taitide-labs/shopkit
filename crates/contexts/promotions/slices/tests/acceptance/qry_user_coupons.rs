use http::StatusCode;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::support::{build_router, get_json, seed_coupon};

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn queries_user_coupons_and_returns_empty_for_new_user(pool: PgPool) {
    seed_coupon(&pool, "VIP50", 20_000, 5_000, 5).await;
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO user_coupons (user_coupon_id, user_id, code, status, claimed_at)
         VALUES ($1, $2, 'VIP50', 'AVAILABLE', NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .execute(&pool)
    .await
    .expect("seed user coupon");

    let app = build_router(pool);
    let (status, body) = get_json(&app, &format!("/api/promotions/users/{user_id}/coupons")).await;
    assert_eq!(status, StatusCode::OK);
    let coupons = body
        .get("coupons")
        .and_then(Value::as_array)
        .expect("coupons array");
    assert_eq!(coupons.len(), 1);
    let first = coupons.first().expect("first coupon");
    assert_eq!(first.get("code").and_then(Value::as_str), Some("VIP50"));
    assert_eq!(
        first.get("status").and_then(Value::as_str),
        Some("AVAILABLE")
    );
    assert_eq!(
        first.get("discount_cents").and_then(Value::as_i64),
        Some(5_000)
    );

    let (empty_status, empty_body) = get_json(
        &app,
        &format!("/api/promotions/users/{}/coupons", Uuid::new_v4()),
    )
    .await;
    assert_eq!(empty_status, StatusCode::OK);
    assert_eq!(
        empty_body
            .get("coupons")
            .and_then(Value::as_array)
            .map(Vec::len),
        Some(0)
    );
}
