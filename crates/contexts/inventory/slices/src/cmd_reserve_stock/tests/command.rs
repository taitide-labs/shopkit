use shopkit_contracts::ports::ReserveStockRequest;
use shopkit_primitives::errors::PrimitiveError;
use shopkit_primitives::ids::OrderId;
use uuid::Uuid;

use super::super::command::{ReserveStockBody, ReserveStockLineDto};

#[test]
fn try_from_converts_valid_body_and_trims_sku() {
    let order_id = Uuid::new_v4();
    let body = ReserveStockBody {
        order_id,
        items: vec![ReserveStockLineDto {
            sku: " SKU-CAM-01 ".to_owned(),
            quantity: 3,
        }],
    };

    let request = ReserveStockRequest::try_from(body).expect("valid reserve stock body");
    assert_eq!(request.order_id, OrderId::from_uuid(order_id));
    assert_eq!(request.items.len(), 1);
    let item = request.items.first().expect("one item");
    assert_eq!(item.sku.as_str(), "SKU-CAM-01");
    assert_eq!(item.quantity.get(), 3);
}

#[test]
fn try_from_rejects_zero_quantity_and_invalid_sku() {
    let zero_qty = ReserveStockBody {
        order_id: Uuid::new_v4(),
        items: vec![ReserveStockLineDto {
            sku: "SKU-CAM-01".to_owned(),
            quantity: 0,
        }],
    };
    assert_eq!(
        ReserveStockRequest::try_from(zero_qty),
        Err(PrimitiveError::ZeroQuantity)
    );

    let bad_sku = ReserveStockBody {
        order_id: Uuid::new_v4(),
        items: vec![ReserveStockLineDto {
            sku: "BAD SKU!".to_owned(),
            quantity: 2,
        }],
    };
    assert!(matches!(
        ReserveStockRequest::try_from(bad_sku),
        Err(PrimitiveError::InvalidSku(_))
    ));
}
