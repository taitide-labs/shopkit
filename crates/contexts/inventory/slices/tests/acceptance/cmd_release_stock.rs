use http::StatusCode;
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::support::{build_router, post_empty, post_json, seed_sku};

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn releases_reserved_stock_idempotently(pool: PgPool) {
    seed_sku(&pool, "SKU-TRIPOD-01", 8, 0).await;
    let app = build_router(pool.clone());

    let (res_status, res_body) = post_json(
        &app,
        "/api/inventory/reservations",
        &json!({
            "order_id": Uuid::new_v4(),
            "items": [{"sku": "SKU-TRIPOD-01", "quantity": 3}]
        }),
    )
    .await;
    assert_eq!(res_status, StatusCode::CREATED);
    let reservation_id = res_body
        .get("reservation_id")
        .and_then(Value::as_str)
        .expect("reservation_id");

    let release_path = format!("/api/inventory/reservations/{reservation_id}/release");
    let (status, body) = post_empty(&app, &release_path).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.get("released").and_then(Value::as_bool), Some(true));

    let (dup_status, _dup_body) = post_empty(&app, &release_path).await;
    assert_eq!(dup_status, StatusCode::OK);

    let row = sqlx::query("SELECT available_qty, reserved_qty FROM sku_stocks WHERE sku = $1")
        .bind("SKU-TRIPOD-01")
        .fetch_one(&pool)
        .await
        .expect("fetch sku");
    assert_eq!(row.get::<i32, _>("available_qty"), 8);
    assert_eq!(row.get::<i32, _>("reserved_qty"), 0);

    let missing_path = format!("/api/inventory/reservations/{}/release", Uuid::new_v4());
    let (not_found_status, _err) = post_empty(&app, &missing_path).await;
    assert_eq!(not_found_status, StatusCode::NOT_FOUND);
}
