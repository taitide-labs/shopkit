use shopkit_infra::state::AppState;
use shopkit_primitives::ids::{OrderId, ReservationId, UserId};
use thiserror::Error;
use uuid::Uuid;

use super::query::{OrderDetailItemView, OrderDetailResponse};

#[derive(Debug, Error)]
pub enum OrderDetailError {
    #[error("订单 {0} 不存在")]
    NotFound(OrderId),
    #[error("订单详情查询错误：{0}")]
    Internal(String),
}

impl From<sqlx::Error> for OrderDetailError {
    fn from(err: sqlx::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

#[derive(sqlx::FromRow)]
struct OrderRow {
    order_id: Uuid,
    user_id: Uuid,
    reservation_id: Uuid,
    coupon_code: Option<String>,
    status: String,
    subtotal_cents: i64,
    discount_cents: i64,
    payable_cents: i64,
}

#[derive(sqlx::FromRow)]
struct OrderItemRow {
    sku: String,
    unit_price_cents: i64,
    quantity: i32,
}

pub async fn handle(
    state: &AppState,
    raw_order_id: Uuid,
) -> Result<OrderDetailResponse, OrderDetailError> {
    let order_id = OrderId::from_uuid(raw_order_id);
    let order = sqlx::query_as::<_, OrderRow>(
        "SELECT order_id, user_id, reservation_id, coupon_code, status, subtotal_cents, discount_cents, payable_cents
         FROM orders
         WHERE order_id = $1",
    )
    .bind(order_id.as_uuid())
    .fetch_optional(&state.pool)
    .await?
    .ok_or(OrderDetailError::NotFound(order_id))?;

    let item_rows = sqlx::query_as::<_, OrderItemRow>(
        "SELECT sku, unit_price_cents, quantity
         FROM order_items
         WHERE order_id = $1
         ORDER BY id ASC",
    )
    .bind(order_id.as_uuid())
    .fetch_all(&state.pool)
    .await?;

    let items = item_rows
        .into_iter()
        .map(|row| OrderDetailItemView {
            sku: row.sku,
            unit_price_cents: row.unit_price_cents,
            quantity: row.quantity,
        })
        .collect();

    Ok(OrderDetailResponse {
        order_id: OrderId::from_uuid(order.order_id),
        user_id: UserId::from_uuid(order.user_id),
        reservation_id: ReservationId::from_uuid(order.reservation_id),
        coupon_code: order.coupon_code,
        status: order.status,
        subtotal_cents: order.subtotal_cents,
        discount_cents: order.discount_cents,
        payable_cents: order.payable_cents,
        items,
    })
}
