use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde::Serialize;
use serde_json::Value;
use shopkit_infra::clock::FixedClock;
use shopkit_infra::event_bus::EventBus;
use shopkit_infra::fakes::{FakeCouponRedemptionPort, FakeStockReservationPort};
use shopkit_infra::routing::SliceDescriptor;
use shopkit_infra::state::AppState;
use sqlx::PgPool;
use time::macros::datetime;
use tower::ServiceExt;
use utoipa_axum::router::OpenApiRouter;

pub struct OrdersTestHarness {
    pub app: Router,
    pub stock_port: Arc<FakeStockReservationPort>,
    pub coupon_port: Arc<FakeCouponRedemptionPort>,
    pub event_bus: EventBus,
}

pub fn build_harness(pool: PgPool, stock: FakeStockReservationPort) -> OrdersTestHarness {
    shopkit_orders_slices::link();
    let stock_port = Arc::new(stock);
    let coupon_port = Arc::new(FakeCouponRedemptionPort::default());
    let event_bus = EventBus::default();
    let clock_instant = datetime!(2026-11-18 0:00:00 UTC);
    let state = AppState::new(
        pool,
        event_bus.clone(),
        Arc::clone(&stock_port) as _,
        Arc::clone(&coupon_port) as _,
        Arc::new(FixedClock::new(clock_instant)),
    );
    let mut api = OpenApiRouter::new();
    for descriptor in inventory::iter::<SliceDescriptor>() {
        if descriptor.context == "orders" {
            api = api.merge((descriptor.router_fn)());
        }
    }
    let (router, _openapi) = api.split_for_parts();
    OrdersTestHarness {
        app: router.with_state(state),
        stock_port,
        coupon_port,
        event_bus,
    }
}

pub async fn post_json<T: Serialize>(app: &Router, uri: &str, payload: &T) -> (StatusCode, Value) {
    let body = serde_json::to_vec(payload).expect("serialize body");
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body))
        .expect("build request");
    let response = app.clone().oneshot(request).await.expect("send request");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

pub async fn post_empty(app: &Router, uri: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .body(Body::empty())
        .expect("build request");
    let response = app.clone().oneshot(request).await.expect("send request");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

pub async fn get_json(app: &Router, uri: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .expect("build request");
    let response = app.clone().oneshot(request).await.expect("send request");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}
