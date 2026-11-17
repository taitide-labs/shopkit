use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde::Serialize;
use serde_json::{Value, json};
use shopkit_infra::clock::FixedClock;
use shopkit_infra::event_bus::EventBus;
use shopkit_infra::routing::RunEnv;
use shopkit_infra::state::AppState;
use shopkit_inventory_slices::port::PgStockReservationPort;
use shopkit_promotions_slices::port::PgCouponRedemptionPort;
use shopkit_server::app::build_app;
use sqlx::{PgPool, Row};
use time::macros::datetime;
use tower::ServiceExt;
use uuid::Uuid;

fn build_e2e_app(pool: PgPool) -> Router {
    let clock_instant = datetime!(2026-11-18 0:00:00 UTC);
    let state = AppState::new(
        pool.clone(),
        EventBus::default(),
        Arc::new(PgStockReservationPort::new(pool.clone())),
        Arc::new(PgCouponRedemptionPort::new(pool)),
        Arc::new(FixedClock::new(clock_instant)),
    );
    build_app(state, RunEnv::Production)
}

async fn send_json<T: Serialize>(
    app: &Router,
    method: &str,
    uri: &str,
    payload: Option<&T>,
) -> (StatusCode, Value) {
    let body = payload
        .map(|value| Body::from(serde_json::to_vec(value).expect("serialize")))
        .unwrap_or_else(Body::empty);
    let mut builder = Request::builder().method(method).uri(uri);
    if payload.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    let request = builder.body(body).expect("build request");
    let response = app.clone().oneshot(request).await.expect("send request");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "wip"]
async fn full_checkout_query_and_cancel_journey_across_contexts(pool: PgPool) {
    sqlx::query(
        "INSERT INTO sku_stocks (sku, available_qty, reserved_qty, updated_at)
         VALUES ('SKU-E2E-1', 10, 0, NOW())",
    )
    .execute(&pool)
    .await
    .expect("seed sku");
    sqlx::query(
        "INSERT INTO coupons (code, min_spend_cents, discount_cents, total_quota, claimed_count, created_at)
         VALUES ('E2E20', 10000, 2000, 5, 0, NOW())",
    )
    .execute(&pool)
    .await
    .expect("seed coupon");

    let app = build_e2e_app(pool.clone());
    let user_id = Uuid::new_v4();

    let (claim_status, _claim_body) = send_json(
        &app,
        "POST",
        "/api/promotions/coupons/E2E20/claim",
        Some(&json!({ "user_id": user_id })),
    )
    .await;
    assert_eq!(claim_status, StatusCode::CREATED);

    let (place_status, place_body) = send_json(
        &app,
        "POST",
        "/api/orders",
        Some(&json!({
            "user_id": user_id,
            "coupon_code": "E2E20",
            "items": [{"sku": "SKU-E2E-1", "unit_price_cents": 6000, "quantity": 2}]
        })),
    )
    .await;
    assert_eq!(place_status, StatusCode::CREATED);
    assert_eq!(
        place_body.get("payable_cents").and_then(Value::as_i64),
        Some(10_000)
    );
    let order_id = place_body
        .get("order_id")
        .and_then(Value::as_str)
        .expect("order_id");

    let (stock_status, stock_body) =
        send_json::<Value>(&app, "GET", "/api/inventory/skus/SKU-E2E-1", None).await;
    assert_eq!(stock_status, StatusCode::OK);
    assert_eq!(
        stock_body.get("available_qty").and_then(Value::as_i64),
        Some(8)
    );
    assert_eq!(
        stock_body.get("reserved_qty").and_then(Value::as_i64),
        Some(2)
    );

    let (detail_status, detail_body) =
        send_json::<Value>(&app, "GET", &format!("/api/orders/{order_id}"), None).await;
    assert_eq!(detail_status, StatusCode::OK);
    assert_eq!(
        detail_body.get("payable_cents").and_then(Value::as_i64),
        Some(10_000)
    );

    let (cancel_status, cancel_body) = send_json::<Value>(
        &app,
        "POST",
        &format!("/api/orders/{order_id}/cancel"),
        None,
    )
    .await;
    assert_eq!(cancel_status, StatusCode::OK);
    assert_eq!(
        cancel_body.get("status").and_then(Value::as_str),
        Some("CANCELLED")
    );

    let (restored_status, restored_body) =
        send_json::<Value>(&app, "GET", "/api/inventory/skus/SKU-E2E-1", None).await;
    assert_eq!(restored_status, StatusCode::OK);
    assert_eq!(
        restored_body.get("available_qty").and_then(Value::as_i64),
        Some(10)
    );
    assert_eq!(
        restored_body.get("reserved_qty").and_then(Value::as_i64),
        Some(0)
    );

    let (coupons_status, coupons_body) = send_json::<Value>(
        &app,
        "GET",
        &format!("/api/promotions/users/{user_id}/coupons"),
        None,
    )
    .await;
    assert_eq!(coupons_status, StatusCode::OK);
    let coupons = coupons_body
        .get("coupons")
        .and_then(Value::as_array)
        .expect("coupons array");
    let coupon = coupons.first().expect("coupon");
    assert_eq!(
        coupon.get("status").and_then(Value::as_str),
        Some("AVAILABLE")
    );

    let outbox_events: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM outbox_events WHERE aggregate_id = $1")
            .bind(order_id)
            .fetch_one(&pool)
            .await
            .expect("count outbox");
    assert_eq!(outbox_events, 2);

    let (over_status, _over_body) = send_json(
        &app,
        "POST",
        "/api/orders",
        Some(&json!({
            "user_id": user_id,
            "coupon_code": "E2E20",
            "items": [{"sku": "SKU-E2E-1", "unit_price_cents": 6000, "quantity": 99}]
        })),
    )
    .await;
    assert_eq!(over_status, StatusCode::CONFLICT);
    let after_fail_coupon =
        sqlx::query("SELECT status FROM user_coupons WHERE user_id = $1 AND code = 'E2E20'")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("coupon still available");
    assert_eq!(after_fail_coupon.get::<String, _>("status"), "AVAILABLE");
}
