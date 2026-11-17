use shopkit_contracts::events::IntegrationEvent;
use shopkit_contracts::ports::{CouponPortError, StockPortError};
use shopkit_infra::state::AppState;
use shopkit_orders_domain::errors::OrderDomainError;
use shopkit_orders_domain::order_state::{CancelledOrder, OrderStatus};
use shopkit_primitives::ids::{OrderId, ReservationId, UserId};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use super::command::CancelOrderResponse;
use super::workflow;

#[derive(Debug, Error)]
pub enum CancelOrderError {
    #[error("订单 {0} 不存在")]
    NotFound(OrderId),
    #[error(transparent)]
    Domain(#[from] OrderDomainError),
    #[error(transparent)]
    Stock(#[from] StockPortError),
    #[error(transparent)]
    Coupon(#[from] CouponPortError),
    #[error("取消订单存储错误：{0}")]
    Internal(String),
}

impl From<sqlx::Error> for CancelOrderError {
    fn from(err: sqlx::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

#[derive(sqlx::FromRow)]
struct OrderCancelRow {
    user_id: Uuid,
    reservation_id: Uuid,
    coupon_code: Option<String>,
    status: String,
}

pub async fn handle(
    state: &AppState,
    raw_order_id: Uuid,
) -> Result<CancelOrderResponse, CancelOrderError> {
    let order_id = OrderId::from_uuid(raw_order_id);
    let row = fetch_order_for_cancel(&state.pool, order_id).await?;
    let cancelled = workflow::prepare_cancel(order_id, &row.status, state.clock.now())?;
    let user_id = UserId::from_uuid(row.user_id);
    let reservation_id = ReservationId::from_uuid(row.reservation_id);
    let has_coupon = row.coupon_code.is_some();

    let event = workflow::order_cancelled_event(cancelled, user_id, reservation_id);
    persist_cancel_and_outbox(&state.pool, cancelled, &event).await?;
    release_external_resources(state, user_id, order_id, reservation_id, has_coupon).await?;

    state.event_bus.publish(event);
    Ok(workflow::to_response(cancelled))
}

async fn fetch_order_for_cancel(
    pool: &PgPool,
    order_id: OrderId,
) -> Result<OrderCancelRow, CancelOrderError> {
    sqlx::query_as::<_, OrderCancelRow>(
        "SELECT user_id, reservation_id, coupon_code, status
         FROM orders
         WHERE order_id = $1",
    )
    .bind(order_id.as_uuid())
    .fetch_optional(pool)
    .await?
    .ok_or(CancelOrderError::NotFound(order_id))
}

async fn persist_cancel_and_outbox(
    pool: &PgPool,
    cancelled: CancelledOrder,
    event: &IntegrationEvent,
) -> Result<(), CancelOrderError> {
    let mut tx = pool.begin().await?;
    let updated = sqlx::query(
        "UPDATE orders
         SET status = $2, cancelled_at = $3
         WHERE order_id = $1 AND status = $4",
    )
    .bind(cancelled.order_id.as_uuid())
    .bind(OrderStatus::Cancelled.as_str())
    .bind(cancelled.cancelled_at)
    .bind(OrderStatus::Unpaid.as_str())
    .execute(&mut *tx)
    .await?;

    if updated.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(CancelOrderError::Domain(OrderDomainError::AlreadyCancelled));
    }

    let payload =
        serde_json::to_value(event).map_err(|err| CancelOrderError::Internal(err.to_string()))?;
    sqlx::query(
        "INSERT INTO outbox_events (event_id, aggregate_type, aggregate_id, event_type, payload, occurred_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::new_v4())
    .bind("Order")
    .bind(cancelled.order_id.as_uuid().to_string())
    .bind(event.event_type())
    .bind(payload)
    .bind(cancelled.cancelled_at)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

async fn release_external_resources(
    state: &AppState,
    user_id: UserId,
    order_id: OrderId,
    reservation_id: ReservationId,
    has_coupon: bool,
) -> Result<(), CancelOrderError> {
    let stock_res = state.stock_port.release_stock(reservation_id).await;
    let coupon_res = if has_coupon {
        state.coupon_port.release_coupon(user_id, order_id).await
    } else {
        Ok(())
    };
    stock_res?;
    coupon_res?;
    Ok(())
}
