use rust_decimal::Decimal;

use crate::errors::PrimitiveError;
use crate::simple_types::{CouponCode, DiscountRule, Money, Quantity, Sku};

#[test]
fn money_rejects_negative_and_converts_decimal() {
    assert_eq!(
        Money::from_cents(-1),
        Err(PrimitiveError::NegativeMoney(-1))
    );
    let price = Money::from_decimal(Decimal::new(1999, 2)).expect("valid decimal price");
    assert_eq!(price.as_cents(), 1999);
    assert_eq!(price.to_decimal(), Decimal::new(1999, 2));

    let qty = Quantity::try_new(3).expect("non-zero quantity");
    assert_eq!(
        price
            .multiply_quantity(qty)
            .expect("no overflow")
            .as_cents(),
        5997
    );
}

#[test]
fn quantity_sku_and_coupon_validate_boundaries() {
    assert_eq!(Quantity::try_new(0), Err(PrimitiveError::ZeroQuantity));
    assert!(Sku::try_new("   ").is_err());
    assert!(Sku::try_new("sku invalid").is_err());
    assert_eq!(
        Sku::try_new(" SKU-001 ").expect("valid sku").as_str(),
        "SKU-001"
    );

    let code = CouponCode::try_new(" save-20 ").expect("valid coupon code");
    assert_eq!(code.as_str(), "SAVE-20");
    assert!(CouponCode::try_new("BAD CODE!").is_err());
}

#[test]
fn discount_rule_applies_when_threshold_met() {
    let min_spend = Money::from_cents(10_000).expect("valid min spend");
    let discount = Money::from_cents(2_000).expect("valid discount");
    let rule = DiscountRule::try_new(min_spend, discount).expect("valid rule");

    assert_eq!(
        rule.applicable_discount(Money::from_cents(9_999).expect("valid")),
        Money::ZERO
    );
    assert_eq!(rule.applicable_discount(min_spend), discount);
    assert!(DiscountRule::try_new(discount, min_spend).is_err());
}
