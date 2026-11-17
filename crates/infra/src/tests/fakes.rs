use shopkit_contracts::ports::{
    CouponPortError, CouponRedemptionPort, RedeemCouponRequest, ReserveStockItem,
    ReserveStockRequest, StockPortError, StockReservationPort,
};
use shopkit_primitives::ids::{OrderId, UserId};
use shopkit_primitives::simple_types::{CouponCode, DiscountRule, Money, Quantity, Sku};

use crate::fakes::{FakeCouponRedemptionPort, FakeStockReservationPort};

#[tokio::test]
async fn fake_stock_port_supports_reserve_idempotency_and_release() {
    let port = FakeStockReservationPort::with_stock([("SKU-A", 5)]);
    let order_id = OrderId::new_v4();
    let req = ReserveStockRequest {
        order_id,
        items: vec![ReserveStockItem {
            sku: Sku::try_new("SKU-A").expect("valid sku"),
            quantity: Quantity::try_new(2).expect("valid qty"),
        }],
    };

    let res_id = port.reserve_stock(&req).await.expect("reserved");
    assert_eq!(port.available("SKU-A"), 3);
    assert_eq!(port.reserve_stock(&req).await.expect("idempotent"), res_id);
    assert_eq!(port.available("SKU-A"), 3);

    port.release_stock(res_id).await.expect("released");
    assert_eq!(port.available("SKU-A"), 5);

    let over_req = ReserveStockRequest {
        order_id: OrderId::new_v4(),
        items: vec![ReserveStockItem {
            sku: Sku::try_new("SKU-A").expect("valid sku"),
            quantity: Quantity::try_new(10).expect("valid qty"),
        }],
    };
    assert!(matches!(
        port.reserve_stock(&over_req).await,
        Err(StockPortError::InsufficientStock { .. })
    ));
}

#[tokio::test]
async fn fake_coupon_port_redeems_and_releases() {
    let port = FakeCouponRedemptionPort::default();
    let user_id = UserId::new_v4();
    let code = CouponCode::try_new("SAVE10").expect("valid code");
    let rule = DiscountRule::try_new(
        Money::from_cents(5000).expect("min"),
        Money::from_cents(1000).expect("disc"),
    )
    .expect("valid rule");
    port.grant_coupon(user_id, code.clone(), rule);

    let order_id = OrderId::new_v4();
    let low_req = RedeemCouponRequest {
        user_id,
        order_id,
        code: code.clone(),
        subtotal: Money::from_cents(4000).expect("low"),
    };
    assert!(matches!(
        port.redeem_coupon(&low_req).await,
        Err(CouponPortError::MinimumSpendNotMet { .. })
    ));

    let valid_req = RedeemCouponRequest {
        user_id,
        order_id,
        code: code.clone(),
        subtotal: Money::from_cents(6000).expect("ok"),
    };
    assert_eq!(
        port.redeem_coupon(&valid_req).await.expect("redeemed"),
        rule
    );
    assert!(!port.is_available(user_id, &code));

    port.release_coupon(user_id, order_id)
        .await
        .expect("released");
    assert!(port.is_available(user_id, &code));
}
