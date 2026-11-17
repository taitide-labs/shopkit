use shopkit_primitives::simple_types::{DiscountRule, Money};

use crate::errors::OrderDomainError;
use crate::order_state::{OrderLine, OrderPricing};

pub fn calculate_pricing(
    items: &[OrderLine],
    discount_rule: Option<DiscountRule>,
) -> Result<OrderPricing, OrderDomainError> {
    if items.is_empty() {
        return Err(OrderDomainError::EmptyItems);
    }

    let mut subtotal = Money::ZERO;
    for item in items {
        let line_total = item.line_total()?;
        subtotal = subtotal
            .checked_add(line_total)
            .map_err(|_err| OrderDomainError::PricingOverflow)?;
    }

    let discount = discount_rule.map_or(Money::ZERO, |rule| rule.applicable_discount(subtotal));
    let payable = subtotal.saturating_sub(discount);

    Ok(OrderPricing {
        subtotal,
        discount,
        payable,
    })
}
