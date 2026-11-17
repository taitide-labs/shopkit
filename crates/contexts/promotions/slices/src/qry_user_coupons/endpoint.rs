use axum::Json;
use axum::extract::{Path, State};
use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::routing::{SliceDescriptor, SliceStatus};
use shopkit_infra::state::AppState;
use shopkit_primitives::ids::UserId;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use super::handler;
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
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserCouponsResponse>, ProblemDetails> {
    let response = handler::handle(&state.pool, UserId::from_uuid(user_id))
        .await
        .map_err(|err| ProblemDetails::internal(err.to_string()))?;
    Ok(Json(response))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(user_coupons_endpoint))
}

inventory::submit! {
    SliceDescriptor {
        context: "promotions",
        name: "qry_user_coupons",
        status: SliceStatus::Active,
        router_fn: router,
    }
}
