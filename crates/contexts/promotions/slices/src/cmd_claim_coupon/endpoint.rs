use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::routing::{SliceDescriptor, SliceStatus};
use shopkit_infra::state::AppState;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::command::{ClaimCouponBody, ClaimCouponInput, ClaimCouponResponse};
use super::handler::{self, ClaimCouponError};

fn map_claim_error(err: ClaimCouponError) -> ProblemDetails {
    match err {
        ClaimCouponError::NotFound => {
            ProblemDetails::not_found("COUPON_NOT_FOUND", "优惠券批次不存在")
        }
        ClaimCouponError::AlreadyClaimed => {
            ProblemDetails::conflict("COUPON_ALREADY_CLAIMED", "用户已领取该优惠券")
        }
        ClaimCouponError::QuotaExhausted => {
            ProblemDetails::conflict("COUPON_QUOTA_EXHAUSTED", "优惠券库存已耗尽")
        }
        ClaimCouponError::Internal(detail) => ProblemDetails::internal(detail),
    }
}

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
    State(state): State<AppState>,
    Path(code): Path<String>,
    Json(body): Json<ClaimCouponBody>,
) -> Result<(StatusCode, Json<ClaimCouponResponse>), ProblemDetails> {
    let input = ClaimCouponInput::try_new(&code, body)
        .map_err(|err| ProblemDetails::unprocessable("INVALID_COUPON_CODE", err.to_string()))?;
    let response = handler::handle(&state, &input)
        .await
        .map_err(map_claim_error)?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(claim_coupon_endpoint))
}

inventory::submit! {
    SliceDescriptor {
        context: "promotions",
        name: "cmd_claim_coupon",
        status: SliceStatus::Active,
        router_fn: router,
    }
}
