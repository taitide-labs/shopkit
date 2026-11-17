use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::routing::{SliceDescriptor, SliceStatus};
use shopkit_infra::state::AppState;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::command::{PlaceOrderBody, PlaceOrderResponse};

#[utoipa::path(
    post,
    path = "/api/orders",
    request_body = PlaceOrderBody,
    responses(
        (status = 201, description = "下单成功", body = PlaceOrderResponse),
        (status = 404, description = "商品或优惠券不存在", body = ProblemDetails),
        (status = 409, description = "库存不足、优惠券已占用或订单号冲突", body = ProblemDetails),
        (status = 422, description = "下单参数不合法或未满足满减门槛", body = ProblemDetails),
    ),
    tag = "orders"
)]
async fn place_order_endpoint(
    State(_state): State<AppState>,
    Json(_body): Json<PlaceOrderBody>,
) -> Result<(StatusCode, Json<PlaceOrderResponse>), ProblemDetails> {
    Err(ProblemDetails::not_implemented("cmd_place_order"))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(place_order_endpoint))
}

inventory::submit! {
    SliceDescriptor {
        context: "orders",
        name: "cmd_place_order",
        status: SliceStatus::Planned,
        router_fn: router,
    }
}
