use shopkit_primitives::simple_types::{Quantity, Sku};

use crate::errors::InventoryDomainError;
use crate::stock_item::StockLevel;

#[test]
fn stock_level_conserves_total_across_reserve_and_release() {
    let sku = Sku::try_new("SKU-BOOK").expect("valid sku");
    let initial = StockLevel::new(sku, 10, 0);

    let reserved = initial
        .reserve(Quantity::try_new(4).expect("valid"))
        .expect("reserved");
    assert_eq!(reserved.available, 6);
    assert_eq!(reserved.reserved, 4);

    assert!(matches!(
        reserved.reserve(Quantity::try_new(7).expect("valid")),
        Err(InventoryDomainError::InsufficientStock { .. })
    ));

    let released = reserved
        .release(Quantity::try_new(3).expect("valid"))
        .expect("released");
    assert_eq!(released.available, 9);
    assert_eq!(released.reserved, 1);

    assert!(matches!(
        released.release(Quantity::try_new(2).expect("valid")),
        Err(InventoryDomainError::InvalidReleaseAmount { .. })
    ));
}
