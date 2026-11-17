use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::routing::{SliceDescriptor, SliceStatus};
use shopkit_infra::state::AppState;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::command::{ClaimCouponBody, ClaimCouponResponse};

#[utoipa::path(
    post,
    path = "/api/promotions/coupons/{code}/claim",
    params(
        ("code" = String, Path, description = "优惠券批次编码")
    ),
    request_body = ClaimCouponBody,
    responses(
        (status = 201, description = "优惠券申领成功", body = ClaimCouponResponse),
        (status = 404, description = "优惠券批次不存在", body = ProblemDetails),
        (status = 409, description = "优惠券库存耗尽或用户已重复申领", body = ProblemDetails),
        (status = 422, description = "优惠券编码不合法", body = ProblemDetails),
    ),
    tag = "promotions"
)]
async fn claim_coupon_endpoint(
    State(_state): State<AppState>,
    Path(_code): Path<String>,
    Json(_body): Json<ClaimCouponBody>,
) -> Result<(StatusCode, Json<ClaimCouponResponse>), ProblemDetails> {
    Err(ProblemDetails::not_implemented("cmd_claim_coupon"))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(claim_coupon_endpoint))
}

inventory::submit! {
    SliceDescriptor {
        context: "promotions",
        name: "cmd_claim_coupon",
        status: SliceStatus::Planned,
        router_fn: router,
    }
}
