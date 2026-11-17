use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use shopkit_infra::clock::SystemClock;
use shopkit_infra::event_bus::EventBus;
use shopkit_infra::routing::RunEnv;
use shopkit_infra::state::AppState;
use shopkit_inventory_slices::port::PgStockReservationPort;
use shopkit_promotions_slices::port::PgCouponRedemptionPort;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let database_url = "postgres://postgres:postgres@localhost:5432/postgres";
    let pool = PgPoolOptions::new()
        .max_connections(16)
        .connect(database_url)
        .await?;
    let state = AppState::new(
        pool.clone(),
        EventBus::default(),
        Arc::new(PgStockReservationPort::new(pool.clone())),
        Arc::new(PgCouponRedemptionPort::new(pool)),
        Arc::new(SystemClock),
    );
    let app = shopkit_server::app::build_app(state, RunEnv::Development);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
