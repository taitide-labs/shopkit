use http::StatusCode;
use serde_json::Value;
use sqlx::PgPool;

use super::support::{build_router, get_json, seed_sku};

#[sqlx::test(migrations = "../../../../migrations")]
#[ignore = "wip"]
async fn queries_sku_stock_or_returns_404(pool: PgPool) {
    seed_sku(&pool, "SKU-BAG-01", 12, 3).await;
    let app = build_router(pool);

    let (status, body) = get_json(&app, "/api/inventory/skus/SKU-BAG-01").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.get("sku").and_then(Value::as_str), Some("SKU-BAG-01"));
    assert_eq!(body.get("available_qty").and_then(Value::as_i64), Some(12));
    assert_eq!(body.get("reserved_qty").and_then(Value::as_i64), Some(3));

    let (missing_status, _err) = get_json(&app, "/api/inventory/skus/SKU-UNKNOWN").await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
}
