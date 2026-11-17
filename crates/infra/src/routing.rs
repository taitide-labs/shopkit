use utoipa_axum::router::OpenApiRouter;

use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceStatus {
    Planned,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunEnv {
    Development,
    Test,
    Production,
}

pub struct SliceDescriptor {
    pub context: &'static str,
    pub name: &'static str,
    pub status: SliceStatus,
    pub router_fn: fn() -> OpenApiRouter<AppState>,
}

inventory::collect!(SliceDescriptor);
