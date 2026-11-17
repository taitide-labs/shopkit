use shopkit_contracts::ports::{
    ReserveStockItem, ReserveStockRequest, StockPortError, StockReservationPort,
};
use shopkit_inventory_domain::errors::InventoryDomainError;
use shopkit_primitives::ids::{OrderId, ReservationId};
use shopkit_primitives::simple_types::{Quantity, Sku};
use sqlx::{PgPool, Row};

use super::super::{PgStockReservationPort, aggregate_requested_items, domain_error_to_port};

#[test]
fn domain_error_to_port_maps_all_variants() {
    let insufficient = domain_error_to_port(InventoryDomainError::InsufficientStock {
        sku: "SKU-1".to_owned(),
        requested: 5,
        available: 2,
    });
    assert_eq!(
        insufficient,
        StockPortError::InsufficientStock {
            sku: "SKU-1".to_owned(),
            requested: 5,
            available: 2,
        }
    );

    let invalid_release = domain_error_to_port(InventoryDomainError::InvalidReleaseAmount {
        sku: "SKU-1".to_owned(),
        reserved: 1,
        release: 3,
    });
    assert!(matches!(invalid_release, StockPortError::Unexpected(_)));
}

#[test]
fn aggregate_requested_items_merges_duplicates_and_sorts_skus() {
    let items = vec![
        ReserveStockItem {
            sku: Sku::try_new("SKU-Z").expect("valid sku"),
            quantity: Quantity::try_new(2).expect("valid qty"),
        },
        ReserveStockItem {
            sku: Sku::try_new("SKU-A").expect("valid sku"),
            quantity: Quantity::try_new(1).expect("valid qty"),
        },
        ReserveStockItem {
            sku: Sku::try_new("SKU-Z").expect("valid sku"),
            quantity: Quantity::try_new(3).expect("valid qty"),
        },
    ];

    let aggregated = aggregate_requested_items(&items).expect("aggregated");
    let pairs: Vec<(&str, u32)> = aggregated
        .iter()
        .map(|(sku, qty)| (sku.as_str(), *qty))
        .collect();
    assert_eq!(pairs, vec![("SKU-A", 1), ("SKU-Z", 5)]);
}

#[sqlx::test(migrations = "../../../../migrations")]
async fn pg_port_reserves_multi_sku_atomically_and_rolls_back_on_missing_sku(pool: PgPool) {
    sqlx::query(
        "INSERT INTO sku_stocks (sku, available_qty, reserved_qty, updated_at)
         VALUES ('SKU-A', 10, 0, NOW()), ('SKU-B', 5, 0, NOW())",
    )
    .execute(&pool)
    .await
    .expect("seed skus");

    let port = PgStockReservationPort::new(pool.clone());

    let missing_req = ReserveStockRequest {
        order_id: OrderId::new_v4(),
        items: vec![
            ReserveStockItem {
                sku: Sku::try_new("SKU-A").expect("sku a"),
                quantity: Quantity::try_new(3).expect("qty"),
            },
            ReserveStockItem {
                sku: Sku::try_new("SKU-MISSING").expect("sku missing"),
                quantity: Quantity::try_new(1).expect("qty"),
            },
        ],
    };
    assert_eq!(
        port.reserve_stock(&missing_req).await,
        Err(StockPortError::SkuNotFound("SKU-MISSING".to_owned()))
    );

    let after_rollback: i32 =
        sqlx::query_scalar("SELECT available_qty FROM sku_stocks WHERE sku = 'SKU-A'")
            .fetch_one(&pool)
            .await
            .expect("fetch sku a");
    assert_eq!(after_rollback, 10);

    let multi_req = ReserveStockRequest {
        order_id: OrderId::new_v4(),
        items: vec![
            ReserveStockItem {
                sku: Sku::try_new("SKU-B").expect("sku b"),
                quantity: Quantity::try_new(2).expect("qty"),
            },
            ReserveStockItem {
                sku: Sku::try_new("SKU-A").expect("sku a"),
                quantity: Quantity::try_new(4).expect("qty"),
            },
            ReserveStockItem {
                sku: Sku::try_new("SKU-B").expect("sku b"),
                quantity: Quantity::try_new(1).expect("qty"),
            },
        ],
    };
    let reservation_id = port.reserve_stock(&multi_req).await.expect("reserved");
    assert_eq!(
        port.reserve_stock(&multi_req).await.expect("idempotent"),
        reservation_id
    );

    let sku_b =
        sqlx::query("SELECT available_qty, reserved_qty FROM sku_stocks WHERE sku = 'SKU-B'")
            .fetch_one(&pool)
            .await
            .expect("fetch sku b");
    assert_eq!(sku_b.get::<i32, _>("available_qty"), 2);
    assert_eq!(sku_b.get::<i32, _>("reserved_qty"), 3);

    port.release_stock(reservation_id).await.expect("released");
    port.release_stock(reservation_id)
        .await
        .expect("idempotent release");

    let restored_b =
        sqlx::query("SELECT available_qty, reserved_qty FROM sku_stocks WHERE sku = 'SKU-B'")
            .fetch_one(&pool)
            .await
            .expect("fetch restored b");
    assert_eq!(restored_b.get::<i32, _>("available_qty"), 5);
    assert_eq!(restored_b.get::<i32, _>("reserved_qty"), 0);

    let unknown_id = ReservationId::new_v4();
    assert_eq!(
        port.release_stock(unknown_id).await,
        Err(StockPortError::ReservationNotFound(unknown_id))
    );
}
