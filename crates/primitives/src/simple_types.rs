use std::fmt;
use std::num::NonZeroU32;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::errors::PrimitiveError;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
)]
#[serde(transparent)]
pub struct Money(i64);

impl Money {
    pub const ZERO: Self = Self(0);

    pub fn from_cents(cents: i64) -> Result<Self, PrimitiveError> {
        if cents < 0 {
            return Err(PrimitiveError::NegativeMoney(cents));
        }
        Ok(Self(cents))
    }

    pub fn from_decimal(amount: Decimal) -> Result<Self, PrimitiveError> {
        if amount.is_sign_negative() {
            return Err(PrimitiveError::NegativeMoney(-1));
        }
        let mut scaled = amount
            .checked_mul(Decimal::from(100))
            .ok_or(PrimitiveError::MoneyScaleOverflow)?;
        if !scaled.fract().is_zero() {
            return Err(PrimitiveError::MoneyScaleOverflow);
        }
        scaled.rescale(0);
        let cents =
            i64::try_from(scaled.mantissa()).map_err(|_err| PrimitiveError::MoneyScaleOverflow)?;
        Self::from_cents(cents)
    }

    #[must_use]
    pub const fn as_cents(self) -> i64 {
        self.0
    }

    #[must_use]
    pub fn to_decimal(self) -> Decimal {
        Decimal::new(self.0, 2)
    }

    pub fn checked_add(self, rhs: Self) -> Result<Self, PrimitiveError> {
        self.0
            .checked_add(rhs.0)
            .map(Self)
            .ok_or(PrimitiveError::MoneyArithmeticOverflow)
    }

    #[must_use]
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        let diff = self.0.saturating_sub(rhs.0);
        if diff < 0 { Self(0) } else { Self(diff) }
    }

    pub fn multiply_quantity(self, quantity: Quantity) -> Result<Self, PrimitiveError> {
        let factor = i64::from(quantity.get());
        self.0
            .checked_mul(factor)
            .map(Self)
            .ok_or(PrimitiveError::MoneyArithmeticOverflow)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.to_decimal())
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
)]
#[schema(value_type = u32)]
#[serde(transparent)]
pub struct Quantity(NonZeroU32);

impl Quantity {
    pub fn try_new(value: u32) -> Result<Self, PrimitiveError> {
        NonZeroU32::new(value)
            .map(Self)
            .ok_or(PrimitiveError::ZeroQuantity)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct Sku(String);

impl Sku {
    pub fn try_new(raw: &str) -> Result<Self, PrimitiveError> {
        let trimmed = raw.trim();
        let valid = !trimmed.is_empty()
            && trimmed.len() <= 64
            && trimmed
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
        if !valid {
            return Err(PrimitiveError::InvalidSku(raw.to_owned()));
        }
        Ok(Self(trimmed.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Sku {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema)]
#[serde(transparent)]
pub struct CouponCode(String);

impl CouponCode {
    pub fn try_new(raw: &str) -> Result<Self, PrimitiveError> {
        let normalized = raw.trim().to_ascii_uppercase();
        let valid = !normalized.is_empty()
            && normalized.len() <= 32
            && normalized
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'-');
        if !valid {
            return Err(PrimitiveError::InvalidCouponCode(raw.to_owned()));
        }
        Ok(Self(normalized))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CouponCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DiscountRule {
    pub min_spend: Money,
    pub discount: Money,
}

impl DiscountRule {
    pub fn try_new(min_spend: Money, discount: Money) -> Result<Self, PrimitiveError> {
        if discount.as_cents() <= 0 || min_spend < discount {
            return Err(PrimitiveError::InvalidDiscountRule {
                min_spend: min_spend.as_cents(),
                discount: discount.as_cents(),
            });
        }
        Ok(Self {
            min_spend,
            discount,
        })
    }

    #[must_use]
    pub fn applicable_discount(self, subtotal: Money) -> Money {
        if subtotal >= self.min_spend {
            self.discount
        } else {
            Money::ZERO
        }
    }
}
