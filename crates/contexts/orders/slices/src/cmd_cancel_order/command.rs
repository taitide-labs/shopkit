use serde::{Deserialize, Serialize};
use shopkit_primitives::ids::OrderId;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CancelOrderResponse {
    pub order_id: OrderId,
    pub status: String,
}
