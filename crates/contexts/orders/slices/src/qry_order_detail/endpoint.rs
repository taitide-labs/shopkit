use axum::Json;
use axum::extract::{Path, State};
use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::routing::{SliceDescriptor, SliceStatus};
use shopkit_infra::state::AppState;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use super::handler::{self, OrderDetailError};
use super::query::OrderDetailResponse;

fn map_error(err: OrderDetailError) -> ProblemDetails {
    match err {
        OrderDetailError::NotFound(order_id) => {
            ProblemDetails::not_found("ORDER_NOT_FOUND", format!("订单 {order_id} 不存在"))
        }
        OrderDetailError::Internal(msg) => ProblemDetails::internal(msg),
    }
}

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
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<OrderDetailResponse>, ProblemDetails> {
    handler::handle(&state, order_id)
        .await
        .map(Json)
        .map_err(map_error)
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(order_detail_endpoint))
}

inventory::submit! {
    SliceDescriptor {
        context: "orders",
        name: "qry_order_detail",
        status: SliceStatus::Active,
        router_fn: router,
    }
}
