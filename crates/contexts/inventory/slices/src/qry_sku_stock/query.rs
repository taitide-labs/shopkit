use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SkuStockResponse {
    pub sku: String,
    pub available_qty: i32,
    pub reserved_qty: i32,
}
