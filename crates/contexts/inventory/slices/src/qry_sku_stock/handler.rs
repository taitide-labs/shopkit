use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::state::AppState;
use shopkit_primitives::simple_types::Sku;
use sqlx::Row;

use super::query::SkuStockResponse;

pub async fn handle(state: &AppState, raw_sku: &str) -> Result<SkuStockResponse, ProblemDetails> {
    let sku = Sku::try_new(raw_sku)
        .map_err(|err| ProblemDetails::unprocessable("INVALID_SKU", err.to_string()))?;

    let row = sqlx::query(
        "SELECT sku, available_qty, reserved_qty
         FROM sku_stocks
         WHERE sku = $1",
    )
    .bind(sku.as_str())
    .fetch_optional(&state.pool)
    .await
    .map_err(|err| ProblemDetails::internal(err.to_string()))?;

    let Some(row) = row else {
        return Err(ProblemDetails::not_found(
            "SKU_NOT_FOUND",
            format!("商品 SKU {sku} 不存在"),
        ));
    };

    let found_sku: String = row
        .try_get("sku")
        .map_err(|err| ProblemDetails::internal(err.to_string()))?;
    let available_qty: i32 = row
        .try_get("available_qty")
        .map_err(|err| ProblemDetails::internal(err.to_string()))?;
    let reserved_qty: i32 = row
        .try_get("reserved_qty")
        .map_err(|err| ProblemDetails::internal(err.to_string()))?;

    Ok(SkuStockResponse {
        sku: found_sku,
        available_qty,
        reserved_qty,
    })
}
