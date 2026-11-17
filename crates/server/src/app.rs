use axum::Router;
use shopkit_infra::routing::{RunEnv, SliceDescriptor, SliceStatus};
use shopkit_infra::state::AppState;
use utoipa::OpenApi;
use utoipa::openapi::OpenApi as OpenApiSpec;
use utoipa_axum::router::OpenApiRouter;

#[derive(OpenApi)]
#[openapi(info(
    title = "shopkit",
    version = "0.1.0",
    description = "AI-native modular monolith e-commerce reference platform"
))]
struct ApiDoc;

#[must_use]
pub const fn service_name() -> &'static str {
    "shopkit-server"
}

fn assemble_router(env: RunEnv) -> OpenApiRouter<AppState> {
    shopkit_orders_slices::link();
    shopkit_inventory_slices::link();
    shopkit_promotions_slices::link();

    let mut api = OpenApiRouter::with_openapi(ApiDoc::openapi());
    for descriptor in inventory::iter::<SliceDescriptor>() {
        if matches!(env, RunEnv::Production) && descriptor.status == SliceStatus::Planned {
            continue;
        }
        api = api.merge((descriptor.router_fn)());
    }
    api
}

pub fn build_app(state: AppState, env: RunEnv) -> Router {
    let (router, _openapi) = assemble_router(env).split_for_parts();
    router.with_state(state)
}

#[must_use]
pub fn build_openapi(env: RunEnv) -> OpenApiSpec {
    let (_router, openapi) = assemble_router(env).split_for_parts();
    openapi
}
