use shopkit_contracts::events::IntegrationEvent;
use shopkit_contracts::ports::{ReserveStockItem, ReserveStockRequest};
use shopkit_orders_domain::errors::OrderDomainError;
use shopkit_orders_domain::order_state::{OrderLine, OrderStatus, UnpaidOrder};
use shopkit_orders_domain::pricing::calculate_pricing;
use shopkit_orders_domain::transitions::place_order;
use shopkit_primitives::ids::ReservationId;
use shopkit_primitives::simple_types::{DiscountRule, Money};
use time::OffsetDateTime;

use super::command::{PlaceOrderInput, PlaceOrderResponse};

pub fn preview_subtotal(items: &[OrderLine]) -> Result<Money, OrderDomainError> {
    let pricing = calculate_pricing(items, None)?;
    Ok(pricing.subtotal)
}

#[must_use]
pub fn build_reserve_stock_request(input: &PlaceOrderInput) -> ReserveStockRequest {
    let items = input
        .items
        .iter()
        .map(|line| ReserveStockItem {
            sku: line.sku.clone(),
            quantity: line.quantity,
        })
        .collect();
    ReserveStockRequest {
        order_id: input.order_id,
        items,
    }
}

pub fn finalize_unpaid_order(
    input: PlaceOrderInput,
    reservation_id: ReservationId,
    discount_rule: Option<DiscountRule>,
    created_at: OffsetDateTime,
) -> Result<UnpaidOrder, OrderDomainError> {
    place_order(
        input.order_id,
        input.user_id,
        reservation_id,
        input.coupon_code,
        input.items,
        discount_rule,
        created_at,
    )
}

#[must_use]
pub fn order_placed_event(order: &UnpaidOrder) -> IntegrationEvent {
    IntegrationEvent::OrderPlaced {
        order_id: order.order_id,
        user_id: order.user_id,
        reservation_id: order.reservation_id,
        payable_cents: order.pricing.payable,
        occurred_at: order.created_at,
    }
}

#[must_use]
pub fn to_response(order: &UnpaidOrder) -> PlaceOrderResponse {
    PlaceOrderResponse {
        order_id: order.order_id,
        reservation_id: order.reservation_id,
        status: OrderStatus::Unpaid.as_str().to_owned(),
        subtotal_cents: order.pricing.subtotal.as_cents(),
        discount_cents: order.pricing.discount.as_cents(),
        payable_cents: order.pricing.payable.as_cents(),
    }
}
