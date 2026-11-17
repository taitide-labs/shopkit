use axum::Json;
use axum::extract::{Path, State};
use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::routing::{SliceDescriptor, SliceStatus};
use shopkit_infra::state::AppState;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::query::SkuStockResponse;

#[utoipa::path(
    get,
    path = "/api/inventory/skus/{sku}",
    params(
        ("sku" = String, Path, description = "商品 SKU 编码")
    ),
    responses(
        (status = 200, description = "库存查询成功", body = SkuStockResponse),
        (status = 404, description = "商品 SKU 不存在", body = ProblemDetails),
        (status = 422, description = "SKU 参数不合法", body = ProblemDetails),
    ),
    tag = "inventory"
)]
async fn sku_stock_endpoint(
    State(state): State<AppState>,
    Path(sku): Path<String>,
) -> Result<Json<SkuStockResponse>, ProblemDetails> {
    super::handler::handle(&state, &sku).await.map(Json)
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(sku_stock_endpoint))
}

inventory::submit! {
    SliceDescriptor {
        context: "inventory",
        name: "qry_sku_stock",
        status: SliceStatus::Active,
        router_fn: router,
    }
}
