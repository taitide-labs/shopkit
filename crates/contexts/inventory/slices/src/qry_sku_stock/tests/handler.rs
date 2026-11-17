use std::sync::Arc;

use shopkit_infra::clock::FixedClock;
use shopkit_infra::event_bus::EventBus;
use shopkit_infra::fakes::{FakeCouponRedemptionPort, FakeStockReservationPort};
use shopkit_infra::state::AppState;
use sqlx::PgPool;
use time::macros::datetime;

use super::super::handler::handle;

#[sqlx::test(migrations = "../../../../migrations")]
async fn rejects_invalid_sku_with_422(pool: PgPool) {
    let state = AppState::new(
        pool,
        EventBus::default(),
        Arc::new(FakeStockReservationPort::default()),
        Arc::new(FakeCouponRedemptionPort::default()),
        Arc::new(FixedClock::new(datetime!(2026-11-18 0:00:00 UTC))),
    );

    let err = handle(&state, "INVALID SKU!")
        .await
        .expect_err("invalid sku must fail");
    assert_eq!(err.status, 422);
}
