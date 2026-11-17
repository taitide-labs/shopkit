use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InventoryDomainError {
    #[error("SKU {sku} 可用库存不足：需要 {requested}，可用 {available}")]
    InsufficientStock {
        sku: String,
        requested: u32,
        available: u32,
    },
    #[error("SKU {sku} 释放数量 {release} 超过当前预占数量 {reserved}")]
    InvalidReleaseAmount {
        sku: String,
        reserved: u32,
        release: u32,
    },
}
