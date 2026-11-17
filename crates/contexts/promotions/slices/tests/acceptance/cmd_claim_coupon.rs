use http::StatusCode;
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::support::{build_router, post_json, seed_coupon};

#[sqlx::test(migrations = "../../../../migrations")]
async fn claims_coupon_and_enforces_quota_and_uniqueness(pool: PgPool) {
    seed_coupon(&pool, "SAVE20", 10_000, 2_000, 1).await;
    let app = build_router(pool.clone());
    let first_user = Uuid::new_v4();
    let second_user = Uuid::new_v4();

    let (status, claimed) = post_json(
        &app,
        "/api/promotions/coupons/save20/claim",
        &json!({ "user_id": first_user }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(claimed.get("code").and_then(Value::as_str), Some("SAVE20"));
    assert_eq!(
        claimed.get("discount_cents").and_then(Value::as_i64),
        Some(2_000)
    );

    let (dup_status, _dup) = post_json(
        &app,
        "/api/promotions/coupons/SAVE20/claim",
        &json!({ "user_id": first_user }),
    )
    .await;
    assert_eq!(dup_status, StatusCode::CONFLICT);

    let (exhausted_status, _exhausted) = post_json(
        &app,
        "/api/promotions/coupons/SAVE20/claim",
        &json!({ "user_id": second_user }),
    )
    .await;
    assert_eq!(exhausted_status, StatusCode::CONFLICT);

    let (missing_status, _missing) = post_json(
        &app,
        "/api/promotions/coupons/NOPE/claim",
        &json!({ "user_id": second_user }),
    )
    .await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);

    let row = sqlx::query("SELECT claimed_count FROM coupons WHERE code = $1")
        .bind("SAVE20")
        .fetch_one(&pool)
        .await
        .expect("fetch coupon");
    assert_eq!(row.get::<i32, _>("claimed_count"), 1);
}
