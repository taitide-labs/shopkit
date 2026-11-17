use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PrimitiveError {
    #[error("金额不能为负数：{0}")]
    NegativeMoney(i64),
    #[error("金额超出合法高精度范围")]
    MoneyScaleOverflow,
    #[error("金额计算溢出")]
    MoneyArithmeticOverflow,
    #[error("数量必须大于 0")]
    ZeroQuantity,
    #[error("SKU 编码不能为空且仅允许字母、数字、短横线或下划线：{0}")]
    InvalidSku(String),
    #[error("优惠券码不能为空且仅允许大写字母、数字或短横线：{0}")]
    InvalidCouponCode(String),
    #[error("满减券门槛 {min_spend} 不能低于优惠金额 {discount}")]
    InvalidDiscountRule { min_spend: i64, discount: i64 },
}
