use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OrderDomainError {
    #[error("订单明细不能为空")]
    EmptyItems,
    #[error("订单金额计算溢出")]
    PricingOverflow,
    #[error("订单已处于取消状态")]
    AlreadyCancelled,
    #[error("当前订单状态 {0} 不允许取消")]
    InvalidStatusTransition(String),
}
