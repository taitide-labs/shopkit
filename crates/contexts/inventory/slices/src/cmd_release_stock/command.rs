use serde::{Deserialize, Serialize};
use shopkit_primitives::ids::ReservationId;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ReleaseStockResponse {
    pub reservation_id: ReservationId,
    pub released: bool,
}
