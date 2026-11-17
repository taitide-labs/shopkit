use http::StatusCode;
use serde_json::Value;
use shopkit_infra::fakes::FakeStockReservationPort;
use sqlx::PgPool;
use uuid::Uuid;

use super::support::{build_harness, get_json};

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn returns_order_detail_with_items_or_404(pool: PgPool) {
    let order_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let reservation_id = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO orders (order_id, user_id, reservation_id, coupon_code, subtotal_cents, discount_cents, payable_cents, currency, status, created_at)
         VALUES ($1, $2, $3, 'PROMO5', 5000, 500, 4500, 'CNY', 'UNPAID', NOW())",
    )
    .bind(order_id)
    .bind(user_id)
    .bind(reservation_id)
    .execute(&pool)
    .await
    .expect("seed order");

    sqlx::query(
        "INSERT INTO order_items (order_id, sku, unit_price_cents, quantity)
         VALUES ($1, 'SKU-LAMP', 2500, 2)",
    )
    .bind(order_id)
    .execute(&pool)
    .await
    .expect("seed item");

    let harness = build_harness(pool, FakeStockReservationPort::default());
    let (status, body) = get_json(&harness.app, &format!("/api/orders/{order_id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.get("status").and_then(Value::as_str), Some("UNPAID"));
    assert_eq!(
        body.get("payable_cents").and_then(Value::as_i64),
        Some(4500)
    );
    let items = body
        .get("items")
        .and_then(Value::as_array)
        .expect("items array");
    assert_eq!(items.len(), 1);
    let item = items.first().expect("first item");
    assert_eq!(item.get("sku").and_then(Value::as_str), Some("SKU-LAMP"));
    assert_eq!(item.get("quantity").and_then(Value::as_i64), Some(2));

    let (missing_status, _missing) =
        get_json(&harness.app, &format!("/api/orders/{}", Uuid::new_v4())).await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
}
