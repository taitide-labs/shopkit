use http::StatusCode;
use serde_json::Value;
use shopkit_contracts::ports::{
    CouponRedemptionPort, RedeemCouponRequest, ReserveStockItem, ReserveStockRequest,
    StockReservationPort,
};
use shopkit_infra::fakes::FakeStockReservationPort;
use shopkit_primitives::ids::{OrderId, UserId};
use shopkit_primitives::simple_types::{CouponCode, DiscountRule, Money, Quantity, Sku};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::support::{build_harness, post_empty};

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn cancels_unpaid_order_releases_resources_and_rejects_duplicate_cancel(pool: PgPool) {
    let harness = build_harness(
        pool.clone(),
        FakeStockReservationPort::with_stock([("SKU-CASE", 6)]),
    );
    let order_id = OrderId::new_v4();
    let user_id = UserId::new_v4();
    let code = CouponCode::try_new("CANCEL10").expect("valid code");
    let rule = DiscountRule::try_new(
        Money::from_cents(5_000).expect("valid"),
        Money::from_cents(1_000).expect("valid"),
    )
    .expect("valid rule");
    harness
        .coupon_port
        .grant_coupon(user_id, code.clone(), rule);

    let reservation_id = harness
        .stock_port
        .reserve_stock(&ReserveStockRequest {
            order_id,
            items: vec![ReserveStockItem {
                sku: Sku::try_new("SKU-CASE").expect("valid sku"),
                quantity: Quantity::try_new(2).expect("valid qty"),
            }],
        })
        .await
        .expect("reserve stock");
    let _redeemed = harness
        .coupon_port
        .redeem_coupon(&RedeemCouponRequest {
            user_id,
            order_id,
            code: code.clone(),
            subtotal: Money::from_cents(8_000).expect("valid"),
        })
        .await
        .expect("redeem coupon");

    sqlx::query(
        "INSERT INTO orders (order_id, user_id, reservation_id, coupon_code, subtotal_cents, discount_cents, payable_cents, currency, status, created_at)
         VALUES ($1, $2, $3, 'CANCEL10', 8000, 1000, 7000, 'CNY', 'UNPAID', NOW())",
    )
    .bind(order_id.as_uuid())
    .bind(user_id.as_uuid())
    .bind(reservation_id.as_uuid())
    .execute(&pool)
    .await
    .expect("seed order");

    let uri = format!("/api/orders/{}/cancel", order_id.as_uuid());
    let (status, body) = post_empty(&harness.app, &uri).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body.get("status").and_then(Value::as_str),
        Some("CANCELLED")
    );

    assert_eq!(harness.stock_port.available("SKU-CASE"), 6);
    assert!(harness.coupon_port.is_available(user_id, &code));

    let order_row = sqlx::query("SELECT status FROM orders WHERE order_id = $1")
        .bind(order_id.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("fetch order");
    assert_eq!(order_row.get::<String, _>("status"), "CANCELLED");

    let outbox_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM outbox_events WHERE aggregate_id = $1 AND event_type = 'OrderCancelled'",
    )
    .bind(order_id.as_uuid().to_string())
    .fetch_one(&pool)
    .await
    .expect("outbox count");
    assert_eq!(outbox_count, 1);

    let (dup_status, _dup_body) = post_empty(&harness.app, &uri).await;
    assert_eq!(dup_status, StatusCode::CONFLICT);

    let (missing_status, _missing_body) = post_empty(
        &harness.app,
        &format!("/api/orders/{}/cancel", Uuid::new_v4()),
    )
    .await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
}
