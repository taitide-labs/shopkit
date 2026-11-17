use std::sync::Arc;

use shopkit_contracts::ports::{Clock, CouponRedemptionPort, StockReservationPort};
use sqlx::PgPool;

use crate::event_bus::EventBus;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub event_bus: EventBus,
    pub stock_port: Arc<dyn StockReservationPort>,
    pub coupon_port: Arc<dyn CouponRedemptionPort>,
    pub clock: Arc<dyn Clock>,
}

impl AppState {
    #[must_use]
    pub fn new(
        pool: PgPool,
        event_bus: EventBus,
        stock_port: Arc<dyn StockReservationPort>,
        coupon_port: Arc<dyn CouponRedemptionPort>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            pool,
            event_bus,
            stock_port,
            coupon_port,
            clock,
        }
    }
}
