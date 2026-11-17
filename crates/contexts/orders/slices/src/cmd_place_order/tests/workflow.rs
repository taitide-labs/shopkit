use shopkit_contracts::events::IntegrationEvent;
use shopkit_orders_domain::errors::OrderDomainError;
use shopkit_orders_domain::order_state::OrderLine;
use shopkit_primitives::ids::{OrderId, ReservationId, UserId};
use shopkit_primitives::simple_types::{CouponCode, DiscountRule, Money, Quantity, Sku};
use time::macros::datetime;

use super::super::command::PlaceOrderInput;
use super::super::workflow::{
    build_reserve_stock_request, finalize_unpaid_order, order_placed_event, preview_subtotal,
    to_response,
};

#[test]
fn preview_subtotal_computes_sum_and_rejects_overflow_or_empty() {
    assert_eq!(preview_subtotal(&[]), Err(OrderDomainError::EmptyItems));

    let items = vec![
        OrderLine {
            sku: Sku::try_new("SKU-A").expect("valid sku"),
            unit_price: Money::from_cents(2_500).expect("valid money"),
            quantity: Quantity::try_new(2).expect("valid qty"),
        },
        OrderLine {
            sku: Sku::try_new("SKU-B").expect("valid sku"),
            unit_price: Money::from_cents(1_200).expect("valid money"),
            quantity: Quantity::try_new(3).expect("valid qty"),
        },
    ];
    let subtotal = preview_subtotal(&items).expect("valid subtotal");
    assert_eq!(subtotal.as_cents(), 8_600);

    let overflow_items = vec![OrderLine {
        sku: Sku::try_new("SKU-MAX").expect("valid sku"),
        unit_price: Money::from_cents(i64::MAX).expect("valid money"),
        quantity: Quantity::try_new(2).expect("valid qty"),
    }];
    assert_eq!(
        preview_subtotal(&overflow_items),
        Err(OrderDomainError::PricingOverflow)
    );
}

#[test]
fn build_reserve_stock_request_and_finalize_order_preserve_fields() {
    let order_id = OrderId::new_v4();
    let user_id = UserId::new_v4();
    let reservation_id = ReservationId::new_v4();
    let coupon_code = CouponCode::try_new("SAVE20").expect("valid code");
    let input = PlaceOrderInput {
        order_id,
        user_id,
        coupon_code: Some(coupon_code),
        items: vec![OrderLine {
            sku: Sku::try_new("SKU-BOOK").expect("valid sku"),
            unit_price: Money::from_cents(6_000).expect("valid money"),
            quantity: Quantity::try_new(2).expect("valid qty"),
        }],
    };

    let stock_req = build_reserve_stock_request(&input);
    assert_eq!(stock_req.order_id, order_id);
    assert_eq!(stock_req.items.len(), 1);
    let item = stock_req.items.first().expect("first stock item");
    assert_eq!(item.sku.as_str(), "SKU-BOOK");
    assert_eq!(item.quantity.get(), 2);

    let rule = DiscountRule::try_new(
        Money::from_cents(10_000).expect("min spend"),
        Money::from_cents(2_000).expect("discount"),
    )
    .expect("valid rule");
    let created_at = datetime!(2026-11-18 0:00:00 UTC);
    let order =
        finalize_unpaid_order(input, reservation_id, Some(rule), created_at).expect("unpaid order");

    assert_eq!(order.pricing.subtotal.as_cents(), 12_000);
    assert_eq!(order.pricing.discount.as_cents(), 2_000);
    assert_eq!(order.pricing.payable.as_cents(), 10_000);

    let response = to_response(&order);
    assert_eq!(response.order_id, order_id);
    assert_eq!(response.reservation_id, reservation_id);
    assert_eq!(response.status, "UNPAID");
    assert_eq!(response.payable_cents, 10_000);

    let event = order_placed_event(&order);
    assert_eq!(
        event,
        IntegrationEvent::OrderPlaced {
            order_id,
            user_id,
            reservation_id,
            payable_cents: Money::from_cents(10_000).expect("payable"),
            occurred_at: created_at,
        }
    );
}
