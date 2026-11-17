use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};
use shopkit_primitives::ids::{OrderId, ReservationId, UserId};
use shopkit_primitives::simple_types::{CouponCode, DiscountRule, Money, Quantity, Sku};
use thiserror::Error;
use time::OffsetDateTime;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Clock: Send + Sync {
    fn now(&self) -> OffsetDateTime;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReserveStockItem {
    pub sku: Sku,
    pub quantity: Quantity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReserveStockRequest {
    pub order_id: OrderId,
    pub items: Vec<ReserveStockItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StockPortError {
    #[error("SKU {sku} 库存不足：需要 {requested}，当前可用 {available}")]
    InsufficientStock {
        sku: String,
        requested: u32,
        available: u32,
    },
    #[error("SKU {0} 不存在")]
    SkuNotFound(String),
    #[error("预占凭证 {0} 不存在")]
    ReservationNotFound(ReservationId),
    #[error("库存端口存储错误：{0}")]
    Unexpected(String),
}

pub trait StockReservationPort: Send + Sync {
    fn reserve_stock<'a>(
        &'a self,
        request: &'a ReserveStockRequest,
    ) -> BoxFuture<'a, Result<ReservationId, StockPortError>>;

    fn release_stock(
        &self,
        reservation_id: ReservationId,
    ) -> BoxFuture<'_, Result<(), StockPortError>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedeemCouponRequest {
    pub user_id: UserId,
    pub order_id: OrderId,
    pub code: CouponCode,
    pub subtotal: Money,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CouponPortError {
    #[error("用户未领取该优惠券")]
    CouponNotOwned,
    #[error("优惠券已被使用")]
    CouponAlreadyUsed,
    #[error("未达到优惠券满减门槛，需满 {min_spend_cents} 分")]
    MinimumSpendNotMet { min_spend_cents: i64 },
    #[error("优惠券端口存储错误：{0}")]
    Unexpected(String),
}

pub trait CouponRedemptionPort: Send + Sync {
    fn redeem_coupon<'a>(
        &'a self,
        request: &'a RedeemCouponRequest,
    ) -> BoxFuture<'a, Result<DiscountRule, CouponPortError>>;

    fn release_coupon(
        &self,
        user_id: UserId,
        order_id: OrderId,
    ) -> BoxFuture<'_, Result<(), CouponPortError>>;
}
