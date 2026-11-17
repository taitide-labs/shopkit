use shopkit_primitives::simple_types::{Quantity, Sku};

use crate::errors::InventoryDomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockLevel {
    pub sku: Sku,
    pub available: u32,
    pub reserved: u32,
}

impl StockLevel {
    #[must_use]
    pub const fn new(sku: Sku, available: u32, reserved: u32) -> Self {
        Self {
            sku,
            available,
            reserved,
        }
    }

    pub fn reserve(&self, quantity: Quantity) -> Result<Self, InventoryDomainError> {
        let req = quantity.get();
        if self.available < req {
            return Err(InventoryDomainError::InsufficientStock {
                sku: self.sku.as_str().to_owned(),
                requested: req,
                available: self.available,
            });
        }
        Ok(Self {
            sku: self.sku.clone(),
            available: self.available - req,
            reserved: self.reserved.saturating_add(req),
        })
    }

    pub fn release(&self, quantity: Quantity) -> Result<Self, InventoryDomainError> {
        let rel = quantity.get();
        if self.reserved < rel {
            return Err(InventoryDomainError::InvalidReleaseAmount {
                sku: self.sku.as_str().to_owned(),
                reserved: self.reserved,
                release: rel,
            });
        }
        Ok(Self {
            sku: self.sku.clone(),
            available: self.available.saturating_add(rel),
            reserved: self.reserved - rel,
        })
    }
}
