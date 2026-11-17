use http::StatusCode;
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::support::{build_router, post_json, seed_sku};

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn reserves_available_stock_idempotently(pool: PgPool) {
    seed_sku(&pool, "SKU-CAM-01", 10, 0).await;
    let app = build_router(pool.clone());
    let order_id = Uuid::new_v4();
    let payload = json!({
        "order_id": order_id,
        "items": [{"sku": "SKU-CAM-01", "quantity": 4}]
    });

    let (status, first) = post_json(&app, "/api/inventory/reservations", &payload).await;
    assert_eq!(status, StatusCode::CREATED);
    let reservation_id = first
        .get("reservation_id")
        .and_then(Value::as_str)
        .expect("reservation_id");

    let (dup_status, second) = post_json(&app, "/api/inventory/reservations", &payload).await;
    assert_eq!(dup_status, StatusCode::CREATED);
    assert_eq!(
        second.get("reservation_id").and_then(Value::as_str),
        Some(reservation_id)
    );

    let row = sqlx::query("SELECT available_qty, reserved_qty FROM sku_stocks WHERE sku = $1")
        .bind("SKU-CAM-01")
        .fetch_one(&pool)
        .await
        .expect("fetch sku");
    assert_eq!(row.get::<i32, _>("available_qty"), 6);
    assert_eq!(row.get::<i32, _>("reserved_qty"), 4);
}

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn rejects_insufficient_stock_and_invalid_quantity(pool: PgPool) {
    seed_sku(&pool, "SKU-LENS-01", 2, 0).await;
    let app = build_router(pool.clone());

    let (conflict_status, _body) = post_json(
        &app,
        "/api/inventory/reservations",
        &json!({
            "order_id": Uuid::new_v4(),
            "items": [{"sku": "SKU-LENS-01", "quantity": 5}]
        }),
    )
    .await;
    assert_eq!(conflict_status, StatusCode::CONFLICT);

    let (invalid_status, _err) = post_json(
        &app,
        "/api/inventory/reservations",
        &json!({
            "order_id": Uuid::new_v4(),
            "items": [{"sku": "SKU-LENS-01", "quantity": 0}]
        }),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);

    let row = sqlx::query("SELECT available_qty FROM sku_stocks WHERE sku = $1")
        .bind("SKU-LENS-01")
        .fetch_one(&pool)
        .await
        .expect("fetch sku");
    assert_eq!(row.get::<i32, _>("available_qty"), 2);
}
