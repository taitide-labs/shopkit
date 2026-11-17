use shopkit_contracts::ports::StockPortError;
use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::state::AppState;
use shopkit_primitives::ids::ReservationId;
use uuid::Uuid;

use super::command::ReleaseStockResponse;

pub async fn handle(
    state: &AppState,
    reservation_id: Uuid,
) -> Result<ReleaseStockResponse, ProblemDetails> {
    let id = ReservationId::from_uuid(reservation_id);
    state
        .stock_port
        .release_stock(id)
        .await
        .map_err(map_stock_port_error)?;
    Ok(ReleaseStockResponse {
        reservation_id: id,
        released: true,
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
