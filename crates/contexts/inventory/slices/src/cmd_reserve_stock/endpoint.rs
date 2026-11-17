use axum::Json;
use axum::extract::State;
use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::routing::{SliceDescriptor, SliceStatus};
use shopkit_infra::state::AppState;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::command::{ReserveStockBody, ReserveStockResponse};

#[utoipa::path(
    post,
    path = "/api/inventory/reservations",
    request_body = ReserveStockBody,
    responses(
        (status = 201, description = "库存预占成功", body = ReserveStockResponse),
        (status = 404, description = "商品 SKU 不存在", body = ProblemDetails),
        (status = 409, description = "可用库存不足", body = ProblemDetails),
        (status = 422, description = "预占请求参数不合法", body = ProblemDetails),
    ),
    tag = "inventory"
)]
async fn reserve_stock_endpoint(
    State(_state): State<AppState>,
    Json(_body): Json<ReserveStockBody>,
) -> Result<(axum::http::StatusCode, Json<ReserveStockResponse>), ProblemDetails> {
    Err(ProblemDetails::not_implemented("cmd_reserve_stock"))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(reserve_stock_endpoint))
}

inventory::submit! {
    SliceDescriptor {
        context: "inventory",
        name: "cmd_reserve_stock",
        status: SliceStatus::Planned,
        router_fn: router,
    }
}
