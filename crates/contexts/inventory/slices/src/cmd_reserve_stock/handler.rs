use shopkit_contracts::ports::{ReserveStockRequest, StockPortError};
use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::state::AppState;

use super::command::{ReserveStockBody, ReserveStockResponse};

pub async fn handle(
    state: &AppState,
    body: ReserveStockBody,
) -> Result<ReserveStockResponse, ProblemDetails> {
    if body.items.is_empty() {
        return Err(ProblemDetails::unprocessable(
            "EMPTY_RESERVATION_ITEMS",
            "预占库存明细不能为空",
        ));
    }
    let request = ReserveStockRequest::try_from(body).map_err(|err| {
        ProblemDetails::unprocessable("INVALID_RESERVE_STOCK_INPUT", err.to_string())
    })?;
    let order_id = request.order_id;
    let reservation_id = state
        .stock_port
        .reserve_stock(&request)
        .await
        .map_err(map_stock_port_error)?;
    Ok(ReserveStockResponse {
        reservation_id,
        order_id,
    })
}

fn map_stock_port_error(err: StockPortError) -> ProblemDetails {
    match err {
        StockPortError::InsufficientStock {
            sku,
            requested,
            available,
        } => ProblemDetails::conflict(
            "INSUFFICIENT_STOCK",
            format!("SKU {sku} 库存不足：需要 {requested}，当前可用 {available}"),
        ),
        StockPortError::SkuNotFound(sku) => {
            ProblemDetails::not_found("SKU_NOT_FOUND", format!("商品 SKU {sku} 不存在"))
        }
        StockPortError::ReservationNotFound(id) => {
            ProblemDetails::not_found("RESERVATION_NOT_FOUND", format!("预占凭证 {id} 不存在"))
        }
        StockPortError::Unexpected(detail) => ProblemDetails::internal(detail),
    }
}
