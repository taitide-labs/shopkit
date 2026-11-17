use shopkit_orders_domain::errors::OrderDomainError;
use shopkit_primitives::errors::PrimitiveError;
use shopkit_primitives::simple_types::CouponCode;
use uuid::Uuid;

use super::super::command::{
    ParsePlaceOrderError, PlaceOrderBody, PlaceOrderInput, PlaceOrderItemDto,
};

#[test]
fn try_from_parses_valid_body_and_normalizes_coupon() {
    let explicit_order_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let body = PlaceOrderBody {
        order_id: Some(explicit_order_id),
        user_id,
        coupon_code: Some(" save-20 ".to_owned()),
        items: vec![PlaceOrderItemDto {
            sku: " SKU-BOOK ".to_owned(),
            unit_price_cents: 6_000,
            quantity: 2,
        }],
    };

    let input = PlaceOrderInput::try_from(body).expect("valid place order body");
    assert_eq!(input.order_id.as_uuid(), explicit_order_id);
    assert_eq!(input.user_id.as_uuid(), user_id);
    assert_eq!(
        input.coupon_code.as_ref().map(CouponCode::as_str),
        Some("SAVE-20")
    );
    assert_eq!(input.items.len(), 1);
    let item = input.items.first().expect("first line");
    assert_eq!(item.sku.as_str(), "SKU-BOOK");
    assert_eq!(item.unit_price.as_cents(), 6_000);
    assert_eq!(item.quantity.get(), 2);
}

#[test]
fn try_from_generates_order_id_when_omitted() {
    let body = PlaceOrderBody {
        order_id: None,
        user_id: Uuid::new_v4(),
        coupon_code: None,
        items: vec![PlaceOrderItemDto {
            sku: "SKU-1".to_owned(),
            unit_price_cents: 100,
            quantity: 1,
        }],
    };

    let input = PlaceOrderInput::try_from(body).expect("valid place order body");
    assert!(input.coupon_code.is_none());
    assert_ne!(input.order_id.as_uuid(), Uuid::nil());
}

#[test]
fn try_from_rejects_empty_items_and_invalid_primitives() {
    let user_id = Uuid::new_v4();

    let empty_err = PlaceOrderInput::try_from(PlaceOrderBody {
        order_id: None,
        user_id,
        coupon_code: None,
        items: Vec::new(),
    });
    assert_eq!(
        empty_err,
        Err(ParsePlaceOrderError::Domain(OrderDomainError::EmptyItems))
    );

    let neg_price_err = PlaceOrderInput::try_from(PlaceOrderBody {
        order_id: None,
        user_id,
        coupon_code: None,
        items: vec![PlaceOrderItemDto {
            sku: "SKU-1".to_owned(),
            unit_price_cents: -1,
            quantity: 1,
        }],
    });
    assert_eq!(
        neg_price_err,
        Err(ParsePlaceOrderError::Primitive(
            PrimitiveError::NegativeMoney(-1)
        ))
    );

    let zero_qty_err = PlaceOrderInput::try_from(PlaceOrderBody {
        order_id: None,
        user_id,
        coupon_code: None,
        items: vec![PlaceOrderItemDto {
            sku: "SKU-1".to_owned(),
            unit_price_cents: 100,
            quantity: 0,
        }],
    });
    assert_eq!(
        zero_qty_err,
        Err(ParsePlaceOrderError::Primitive(
            PrimitiveError::ZeroQuantity
        ))
    );

    let bad_sku_err = PlaceOrderInput::try_from(PlaceOrderBody {
        order_id: None,
        user_id,
        coupon_code: None,
        items: vec![PlaceOrderItemDto {
            sku: "bad sku!".to_owned(),
            unit_price_cents: 100,
            quantity: 1,
        }],
    });
    assert!(matches!(
        bad_sku_err,
        Err(ParsePlaceOrderError::Primitive(PrimitiveError::InvalidSku(
            _
        )))
    ));

    let bad_coupon_err = PlaceOrderInput::try_from(PlaceOrderBody {
        order_id: None,
        user_id,
        coupon_code: Some("bad code!".to_owned()),
        items: vec![PlaceOrderItemDto {
            sku: "SKU-1".to_owned(),
            unit_price_cents: 100,
            quantity: 1,
        }],
    });
    assert!(matches!(
        bad_coupon_err,
        Err(ParsePlaceOrderError::Primitive(
            PrimitiveError::InvalidCouponCode(_)
        ))
    ));
}
