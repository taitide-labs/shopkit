use axum::Json;
use axum::extract::{Path, State};
use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::routing::{SliceDescriptor, SliceStatus};
use shopkit_infra::state::AppState;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use super::command::CancelOrderResponse;

#[utoipa::path(
    post,
    path = "/api/orders/{order_id}/cancel",
    params(
        ("order_id" = Uuid, Path, description = "订单 ID")
    ),
    responses(
        (status = 200, description = "订单取消成功并释放库存与优惠券", body = CancelOrderResponse),
        (status = 404, description = "订单不存在", body = ProblemDetails),
        (status = 409, description = "订单已取消或状态不允许取消", body = ProblemDetails),
    ),
    tag = "orders"
)]
async fn cancel_order_endpoint(
    State(_state): State<AppState>,
    Path(_order_id): Path<Uuid>,
) -> Result<Json<CancelOrderResponse>, ProblemDetails> {
    Err(ProblemDetails::not_implemented("cmd_cancel_order"))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(cancel_order_endpoint))
}

inventory::submit! {
    SliceDescriptor {
        context: "orders",
        name: "cmd_cancel_order",
        status: SliceStatus::Planned,
        router_fn: router,
    }
}
