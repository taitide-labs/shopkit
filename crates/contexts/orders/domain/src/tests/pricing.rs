use shopkit_primitives::simple_types::{DiscountRule, Money, Quantity, Sku};

use crate::errors::OrderDomainError;
use crate::order_state::OrderLine;
use crate::pricing::calculate_pricing;

#[test]
fn pricing_rejects_empty_items_and_computes_discount() {
    assert_eq!(
        calculate_pricing(&[], None),
        Err(OrderDomainError::EmptyItems)
    );

    let lines = vec![
        OrderLine {
            sku: Sku::try_new("SKU-BOOK").expect("valid sku"),
            unit_price: Money::from_cents(4_500).expect("valid"),
            quantity: Quantity::try_new(2).expect("valid"),
        },
        OrderLine {
            sku: Sku::try_new("SKU-PEN").expect("valid sku"),
            unit_price: Money::from_cents(1_500).expect("valid"),
            quantity: Quantity::try_new(1).expect("valid"),
        },
    ];
    let rule = DiscountRule::try_new(
        Money::from_cents(10_000).expect("min"),
        Money::from_cents(2_000).expect("disc"),
    )
    .expect("valid rule");

    let pricing = calculate_pricing(&lines, Some(rule)).expect("priced");
    assert_eq!(pricing.subtotal.as_cents(), 10_500);
    assert_eq!(pricing.discount.as_cents(), 2_000);
    assert_eq!(pricing.payable.as_cents(), 8_500);
}
