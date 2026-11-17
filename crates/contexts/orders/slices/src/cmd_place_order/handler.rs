use shopkit_contracts::events::IntegrationEvent;
use shopkit_contracts::ports::{CouponPortError, RedeemCouponRequest, StockPortError};
use shopkit_infra::state::AppState;
use shopkit_orders_domain::errors::OrderDomainError;
use shopkit_orders_domain::order_state::{OrderStatus, UnpaidOrder};
use shopkit_primitives::ids::{OrderId, ReservationId, UserId};
use shopkit_primitives::simple_types::{CouponCode, DiscountRule, Money};
use sqlx::{PgPool, Postgres, Transaction};
use thiserror::Error;
use uuid::Uuid;

use super::command::{ParsePlaceOrderError, PlaceOrderBody, PlaceOrderInput, PlaceOrderResponse};
use super::workflow;

#[derive(Debug, Error)]
pub enum PlaceOrderError {
    #[error(transparent)]
    Parse(#[from] ParsePlaceOrderError),
    #[error(transparent)]
    Domain(#[from] OrderDomainError),
    #[error(transparent)]
    Stock(#[from] StockPortError),
    #[error(transparent)]
    Coupon(#[from] CouponPortError),
    #[error("订单 {0} 已存在")]
    OrderAlreadyExists(OrderId),
    #[error("订单存储错误：{0}")]
    Internal(String),
}

impl From<sqlx::Error> for PlaceOrderError {
    fn from(err: sqlx::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

pub async fn handle(
    state: &AppState,
    body: PlaceOrderBody,
) -> Result<PlaceOrderResponse, PlaceOrderError> {
    let has_explicit_order_id = body.order_id.is_some();
    let input = PlaceOrderInput::try_from(body)?;
    let subtotal = workflow::preview_subtotal(&input.items)?;
    if has_explicit_order_id && order_exists(&state.pool, input.order_id).await? {
        return Err(PlaceOrderError::OrderAlreadyExists(input.order_id));
    }

    let stock_request = workflow::build_reserve_stock_request(&input);
    let reservation_id = state.stock_port.reserve_stock(&stock_request).await?;
    let discount_rule =
        redeem_coupon_or_compensate(state, &input, subtotal, reservation_id).await?;

    let order_id = input.order_id;
    let user_id = input.user_id;
    let coupon_redeemed = discount_rule.is_some();
    let unpaid_order = match workflow::finalize_unpaid_order(
        input,
        reservation_id,
        discount_rule,
        state.clock.now(),
    ) {
        Ok(order) => order,
        Err(err) => {
            compensate_external_resources(
                state,
                user_id,
                order_id,
                reservation_id,
                coupon_redeemed,
            )
            .await;
            return Err(PlaceOrderError::Domain(err));
        }
    };

    let event = workflow::order_placed_event(&unpaid_order);
    if let Err(err) = persist_order_and_outbox(&state.pool, &unpaid_order, &event).await {
        if !matches!(err, PlaceOrderError::OrderAlreadyExists(_)) {
            compensate_external_resources(
                state,
                user_id,
                order_id,
                reservation_id,
                coupon_redeemed,
            )
            .await;
        }
        return Err(err);
    }

    state.event_bus.publish(event);
    Ok(workflow::to_response(&unpaid_order))
}

async fn order_exists(pool: &PgPool, order_id: OrderId) -> Result<bool, PlaceOrderError> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM orders WHERE order_id = $1)")
            .bind(order_id.as_uuid())
            .fetch_one(pool)
            .await?;
    Ok(exists)
}

async fn redeem_coupon_or_compensate(
    state: &AppState,
    input: &PlaceOrderInput,
    subtotal: Money,
    reservation_id: ReservationId,
) -> Result<Option<DiscountRule>, PlaceOrderError> {
    let Some(code) = input.coupon_code.clone() else {
        return Ok(None);
    };
    let request = RedeemCouponRequest {
        user_id: input.user_id,
        order_id: input.order_id,
        code,
        subtotal,
    };
    match state.coupon_port.redeem_coupon(&request).await {
        Ok(rule) => Ok(Some(rule)),
        Err(err) => {
            let _stock_release = state.stock_port.release_stock(reservation_id).await;
            Err(PlaceOrderError::Coupon(err))
        }
    }
}

async fn compensate_external_resources(
    state: &AppState,
    user_id: UserId,
    order_id: OrderId,
    reservation_id: ReservationId,
    coupon_redeemed: bool,
) {
    if coupon_redeemed {
        let _coupon_release = state.coupon_port.release_coupon(user_id, order_id).await;
    }
    let _stock_release = state.stock_port.release_stock(reservation_id).await;
}

async fn persist_order_and_outbox(
    pool: &PgPool,
    order: &UnpaidOrder,
    event: &IntegrationEvent,
) -> Result<(), PlaceOrderError> {
    let mut tx = pool.begin().await?;
    insert_order_row(&mut tx, order).await?;
    insert_order_items(&mut tx, order).await?;
    insert_outbox_event(&mut tx, order, event).await?;
    tx.commit().await?;
    Ok(())
}

async fn insert_order_row(
    tx: &mut Transaction<'_, Postgres>,
    order: &UnpaidOrder,
) -> Result<(), PlaceOrderError> {
    let result = sqlx::query(
        "INSERT INTO orders (order_id, user_id, reservation_id, coupon_code, subtotal_cents, discount_cents, payable_cents, currency, status, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(order.order_id.as_uuid())
    .bind(order.user_id.as_uuid())
    .bind(order.reservation_id.as_uuid())
    .bind(order.coupon_code.as_ref().map(CouponCode::as_str))
    .bind(order.pricing.subtotal.as_cents())
    .bind(order.pricing.discount.as_cents())
    .bind(order.pricing.payable.as_cents())
    .bind("CNY")
    .bind(OrderStatus::Unpaid.as_str())
    .bind(order.created_at)
    .execute(&mut **tx)
    .await;

    match result {
        Ok(_done) => Ok(()),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Err(PlaceOrderError::OrderAlreadyExists(order.order_id))
        }
        Err(err) => Err(PlaceOrderError::Internal(err.to_string())),
    }
}

async fn insert_order_items(
    tx: &mut Transaction<'_, Postgres>,
    order: &UnpaidOrder,
) -> Result<(), PlaceOrderError> {
    for line in &order.items {
        let quantity = i32::try_from(line.quantity.get())
            .map_err(|err| PlaceOrderError::Internal(err.to_string()))?;
        sqlx::query(
            "INSERT INTO order_items (order_id, sku, unit_price_cents, quantity)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(order.order_id.as_uuid())
        .bind(line.sku.as_str())
        .bind(line.unit_price.as_cents())
        .bind(quantity)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn insert_outbox_event(
    tx: &mut Transaction<'_, Postgres>,
    order: &UnpaidOrder,
    event: &IntegrationEvent,
) -> Result<(), PlaceOrderError> {
    let payload =
        serde_json::to_value(event).map_err(|err| PlaceOrderError::Internal(err.to_string()))?;
    sqlx::query(
        "INSERT INTO outbox_events (event_id, aggregate_type, aggregate_id, event_type, payload, occurred_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::new_v4())
    .bind("Order")
    .bind(order.order_id.as_uuid().to_string())
    .bind(event.event_type())
    .bind(payload)
    .bind(order.created_at)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
