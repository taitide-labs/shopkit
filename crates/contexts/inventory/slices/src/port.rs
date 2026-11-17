use shopkit_contracts::ports::{
    BoxFuture, ReserveStockRequest, StockPortError, StockReservationPort,
};
use shopkit_inventory_domain::errors::InventoryDomainError;
use shopkit_primitives::ids::ReservationId;
use sqlx::PgPool;

#[must_use]
pub fn domain_error_to_port(err: InventoryDomainError) -> StockPortError {
    match err {
        InventoryDomainError::InsufficientStock {
            sku,
            requested,
            available,
        } => StockPortError::InsufficientStock {
            sku,
            requested,
            available,
        },
        InventoryDomainError::InvalidReleaseAmount {
            sku,
            reserved,
            release,
        } => StockPortError::Unexpected(format!(
            "SKU {sku} 预占数量不一致：当前 {reserved}，释放 {release}"
        )),
    }
}

#[derive(Clone, Debug)]
pub struct PgStockReservationPort {
    pool: PgPool,
}

impl PgStockReservationPort {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[must_use]
    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl StockReservationPort for PgStockReservationPort {
    fn reserve_stock<'a>(
        &'a self,
        _request: &'a ReserveStockRequest,
    ) -> BoxFuture<'a, Result<ReservationId, StockPortError>> {
        Box::pin(async move { Err(StockPortError::Unexpected("planned slice".to_owned())) })
    }

    fn release_stock(
        &self,
        _reservation_id: ReservationId,
    ) -> BoxFuture<'_, Result<(), StockPortError>> {
        Box::pin(async move { Err(StockPortError::Unexpected("planned slice".to_owned())) })
    }
}
