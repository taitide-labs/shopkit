use http::StatusCode;
use serde_json::{Value, json};
use shopkit_infra::fakes::FakeStockReservationPort;
use shopkit_primitives::ids::UserId;
use shopkit_primitives::simple_types::{CouponCode, DiscountRule, Money};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::support::{build_harness, post_json};

#[sqlx::test(migrations = "../../../../migrations")]
async fn places_order_with_coupon_persists_outbox_and_reserves_stock(pool: PgPool) {
    let harness = build_harness(
        pool.clone(),
        FakeStockReservationPort::with_stock([("SKU-BOOK", 10)]),
    );
    let mut rx = harness.event_bus.subscribe();
    let user_id = UserId::new_v4();
    let code = CouponCode::try_new("SAVE20").expect("valid code");
    let rule = DiscountRule::try_new(
        Money::from_cents(10_000).expect("valid"),
        Money::from_cents(2_000).expect("valid"),
    )
    .expect("valid rule");
    harness
        .coupon_port
        .grant_coupon(user_id, code.clone(), rule);

    let (status, body) = post_json(
        &harness.app,
        "/api/orders",
        &json!({
            "user_id": user_id.as_uuid(),
            "coupon_code": "save20",
            "items": [{"sku": "SKU-BOOK", "unit_price_cents": 6_000, "quantity": 2}]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body.get("status").and_then(Value::as_str), Some("UNPAID"));
    assert_eq!(
        body.get("subtotal_cents").and_then(Value::as_i64),
        Some(12_000)
    );
    assert_eq!(
        body.get("discount_cents").and_then(Value::as_i64),
        Some(2_000)
    );
    assert_eq!(
        body.get("payable_cents").and_then(Value::as_i64),
        Some(10_000)
    );

    let order_id = Uuid::parse_str(
        body.get("order_id")
            .and_then(Value::as_str)
            .expect("order_id"),
    )
    .expect("uuid");
    assert_eq!(harness.stock_port.available("SKU-BOOK"), 8);
    assert!(!harness.coupon_port.is_available(user_id, &code));

    let outbox_row = sqlx::query(
        "SELECT event_type FROM outbox_events WHERE aggregate_id = $1 AND event_type = 'OrderPlaced'",
    )
    .bind(order_id.to_string())
    .fetch_one(&pool)
    .await
    .expect("outbox row");
    assert_eq!(outbox_row.get::<String, _>("event_type"), "OrderPlaced");

    let published = rx.try_recv().expect("event published");
    assert_eq!(published.event_type(), "OrderPlaced");
}

#[sqlx::test(migrations = "../../../../migrations")]
async fn compensates_reserved_stock_when_coupon_below_threshold(pool: PgPool) {
    let harness = build_harness(
        pool.clone(),
        FakeStockReservationPort::with_stock([("SKU-PEN", 5)]),
    );
    let user_id = UserId::new_v4();
    let code = CouponCode::try_new("BIG50").expect("valid code");
    let rule = DiscountRule::try_new(
        Money::from_cents(20_000).expect("valid"),
        Money::from_cents(5_000).expect("valid"),
    )
    .expect("valid rule");
    harness.coupon_port.grant_coupon(user_id, code, rule);

    let (status, _err) = post_json(
        &harness.app,
        "/api/orders",
        &json!({
            "user_id": user_id.as_uuid(),
            "coupon_code": "BIG50",
            "items": [{"sku": "SKU-PEN", "unit_price_cents": 2_000, "quantity": 2}]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(harness.stock_port.available("SKU-PEN"), 5);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM orders")
        .fetch_one(&pool)
        .await
        .expect("count orders");
    assert_eq!(count, 0);
}

#[sqlx::test(migrations = "../../../../migrations")]
async fn rejects_empty_items_and_insufficient_stock(pool: PgPool) {
    let harness = build_harness(
        pool,
        FakeStockReservationPort::with_stock([("SKU-RARE", 1)]),
    );
    let user_id = Uuid::new_v4();

    let (empty_status, _empty_err) = post_json(
        &harness.app,
        "/api/orders",
        &json!({
            "user_id": user_id,
            "coupon_code": null,
            "items": []
        }),
    )
    .await;
    assert_eq!(empty_status, StatusCode::UNPROCESSABLE_ENTITY);

    let (conflict_status, _conflict_err) = post_json(
        &harness.app,
        "/api/orders",
        &json!({
            "user_id": user_id,
            "coupon_code": null,
            "items": [{"sku": "SKU-RARE", "unit_price_cents": 1_000, "quantity": 3}]
        }),
    )
    .await;
    assert_eq!(conflict_status, StatusCode::CONFLICT);
}
