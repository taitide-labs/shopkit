use axum::Json;
use axum::extract::{Path, State};
use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::routing::{SliceDescriptor, SliceStatus};
use shopkit_infra::state::AppState;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use super::query::OrderDetailResponse;

#[utoipa::path(
    get,
    path = "/api/orders/{order_id}",
    params(
        ("order_id" = Uuid, Path, description = "订单 ID")
    ),
    responses(
        (status = 200, description = "订单详情查询成功", body = OrderDetailResponse),
        (status = 404, description = "订单不存在", body = ProblemDetails),
    ),
    tag = "orders"
)]
async fn order_detail_endpoint(
    State(_state): State<AppState>,
    Path(_order_id): Path<Uuid>,
) -> Result<Json<OrderDetailResponse>, ProblemDetails> {
    Err(ProblemDetails::not_implemented("qry_order_detail"))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(order_detail_endpoint))
}

inventory::submit! {
    SliceDescriptor {
        context: "orders",
        name: "qry_order_detail",
        status: SliceStatus::Planned,
        router_fn: router,
    }
}
