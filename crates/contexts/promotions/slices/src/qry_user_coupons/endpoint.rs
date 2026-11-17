use axum::Json;
use axum::extract::{Path, State};
use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::routing::{SliceDescriptor, SliceStatus};
use shopkit_infra::state::AppState;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use super::query::UserCouponsResponse;

#[utoipa::path(
    get,
    path = "/api/promotions/users/{user_id}/coupons",
    params(
        ("user_id" = Uuid, Path, description = "用户 ID")
    ),
    responses(
        (status = 200, description = "用户优惠券列表查询成功", body = UserCouponsResponse),
    ),
    tag = "promotions"
)]
async fn user_coupons_endpoint(
    State(_state): State<AppState>,
    Path(_user_id): Path<Uuid>,
) -> Result<Json<UserCouponsResponse>, ProblemDetails> {
    Err(ProblemDetails::not_implemented("qry_user_coupons"))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(user_coupons_endpoint))
}

inventory::submit! {
    SliceDescriptor {
        context: "promotions",
        name: "qry_user_coupons",
        status: SliceStatus::Planned,
        router_fn: router,
    }
}
