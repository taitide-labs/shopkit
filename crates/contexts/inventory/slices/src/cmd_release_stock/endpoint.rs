use axum::Json;
use axum::extract::{Path, State};
use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::routing::{SliceDescriptor, SliceStatus};
use shopkit_infra::state::AppState;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use super::command::ReleaseStockResponse;

#[utoipa::path(
    post,
    path = "/api/inventory/reservations/{reservation_id}/release",
    params(
        ("reservation_id" = Uuid, Path, description = "预占凭证 ID")
    ),
    responses(
        (status = 200, description = "库存释放成功", body = ReleaseStockResponse),
        (status = 404, description = "预占凭证不存在", body = ProblemDetails),
    ),
    tag = "inventory"
)]
async fn release_stock_endpoint(
    State(_state): State<AppState>,
    Path(_reservation_id): Path<Uuid>,
) -> Result<Json<ReleaseStockResponse>, ProblemDetails> {
    Err(ProblemDetails::not_implemented("cmd_release_stock"))
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(release_stock_endpoint))
}

inventory::submit! {
    SliceDescriptor {
        context: "inventory",
        name: "cmd_release_stock",
        status: SliceStatus::Planned,
        router_fn: router,
    }
}
