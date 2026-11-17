use std::collections::BTreeMap;

use shopkit_contracts::ports::{
    BoxFuture, ReserveStockItem, ReserveStockRequest, StockPortError, StockReservationPort,
};
use shopkit_inventory_domain::errors::InventoryDomainError;
use shopkit_inventory_domain::stock_item::StockLevel;
use shopkit_primitives::ids::{OrderId, ReservationId};
use shopkit_primitives::simple_types::{Quantity, Sku};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

#[cfg(test)]
mod tests;

#[must_use]
pub fn domain_error_to_port(err: InventoryDomainError) -> StockPortError {
    match err {
        InventoryDomainError::InsufficientStock {
            sku,
            requested,
            available,
        } => StockPortError::InsufficientStock {
            sku,
            requested,
            available,
        },
        InventoryDomainError::InvalidReleaseAmount {
            sku,
            reserved,
            release,
        } => StockPortError::Unexpected(format!(
            "SKU {sku} 预占数量不一致：当前 {reserved}，释放 {release}"
        )),
    }
}

#[derive(Clone, Debug)]
pub struct PgStockReservationPort {
    pool: PgPool,
}

impl PgStockReservationPort {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    #[must_use]
    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl StockReservationPort for PgStockReservationPort {
    fn reserve_stock<'a>(
        &'a self,
        request: &'a ReserveStockRequest,
    ) -> BoxFuture<'a, Result<ReservationId, StockPortError>> {
        Box::pin(reserve_stock_impl(&self.pool, request))
    }

    fn release_stock(
        &self,
        reservation_id: ReservationId,
    ) -> BoxFuture<'_, Result<(), StockPortError>> {
        Box::pin(release_stock_impl(&self.pool, reservation_id))
    }
}

fn db_error(err: &sqlx::Error) -> StockPortError {
    StockPortError::Unexpected(err.to_string())
}

pub(crate) fn aggregate_requested_items(
    items: &[ReserveStockItem],
) -> Result<BTreeMap<Sku, u32>, StockPortError> {
    let mut aggregated: BTreeMap<Sku, u32> = BTreeMap::new();
    for item in items {
        if let Some(entry) = aggregated.get_mut(&item.sku) {
            *entry = entry.checked_add(item.quantity.get()).ok_or_else(|| {
                StockPortError::Unexpected(format!("SKU {} 预占数量溢出", item.sku))
            })?;
        } else {
            aggregated.insert(item.sku.clone(), item.quantity.get());
        }
    }
    Ok(aggregated)
}

async fn find_reservation_by_order(
    tx: &mut Transaction<'_, Postgres>,
    order_id: OrderId,
) -> Result<Option<ReservationId>, StockPortError> {
    let existing: Option<Uuid> =
        sqlx::query_scalar("SELECT reservation_id FROM stock_reservations WHERE order_id = $1")
            .bind(order_id.as_uuid())
            .fetch_optional(&mut **tx)
            .await
            .map_err(|err| db_error(&err))?;
    Ok(existing.map(ReservationId::from_uuid))
}

async fn find_reservation_by_order_on_pool(
    pool: &PgPool,
    order_id: OrderId,
) -> Result<ReservationId, StockPortError> {
    let existing: Option<Uuid> =
        sqlx::query_scalar("SELECT reservation_id FROM stock_reservations WHERE order_id = $1")
            .bind(order_id.as_uuid())
            .fetch_optional(pool)
            .await
            .map_err(|err| db_error(&err))?;
    existing
        .map(ReservationId::from_uuid)
        .ok_or_else(|| StockPortError::Unexpected(format!("订单 {order_id} 预占记录丢失")))
}

async fn lock_sku_stock(
    tx: &mut Transaction<'_, Postgres>,
    sku: &Sku,
) -> Result<StockLevel, StockPortError> {
    let row = sqlx::query(
        "SELECT available_qty, reserved_qty
         FROM sku_stocks
         WHERE sku = $1
         FOR UPDATE",
    )
    .bind(sku.as_str())
    .fetch_optional(&mut **tx)
    .await
    .map_err(|err| db_error(&err))?;

    let Some(row) = row else {
        return Err(StockPortError::SkuNotFound(sku.as_str().to_owned()));
    };

    let available_raw: i32 = row.try_get("available_qty").map_err(|err| db_error(&err))?;
    let reserved_raw: i32 = row.try_get("reserved_qty").map_err(|err| db_error(&err))?;
    let available = u32::try_from(available_raw).map_err(|_err| {
        StockPortError::Unexpected(format!("SKU {sku} 可用库存异常为负值：{available_raw}"))
    })?;
    let reserved = u32::try_from(reserved_raw).map_err(|_err| {
        StockPortError::Unexpected(format!("SKU {sku} 预占库存异常为负值：{reserved_raw}"))
    })?;
    Ok(StockLevel::new(sku.clone(), available, reserved))
}

async fn update_sku_stock(
    tx: &mut Transaction<'_, Postgres>,
    level: &StockLevel,
) -> Result<(), StockPortError> {
    let next_available = i32::try_from(level.available).map_err(|_err| {
        StockPortError::Unexpected(format!("SKU {} 可用库存超出整数上限", level.sku))
    })?;
    let next_reserved = i32::try_from(level.reserved).map_err(|_err| {
        StockPortError::Unexpected(format!("SKU {} 预占库存超出整数上限", level.sku))
    })?;
    sqlx::query(
        "UPDATE sku_stocks
         SET available_qty = $2, reserved_qty = $3, updated_at = NOW()
         WHERE sku = $1",
    )
    .bind(level.sku.as_str())
    .bind(next_available)
    .bind(next_reserved)
    .execute(&mut **tx)
    .await
    .map_err(|err| db_error(&err))?;
    Ok(())
}

async fn reserve_stock_impl(
    pool: &PgPool,
    request: &ReserveStockRequest,
) -> Result<ReservationId, StockPortError> {
    let needed = aggregate_requested_items(&request.items)?;
    let mut tx = pool.begin().await.map_err(|err| db_error(&err))?;

    if let Some(existing_id) = find_reservation_by_order(&mut tx, request.order_id).await? {
        tx.commit().await.map_err(|err| db_error(&err))?;
        return Ok(existing_id);
    }

    let mut updates = Vec::with_capacity(needed.len());
    let mut stored_items = Vec::with_capacity(needed.len());
    for (sku, requested) in &needed {
        let current = lock_sku_stock(&mut tx, sku).await?;
        let quantity = Quantity::try_new(*requested)
            .map_err(|err| StockPortError::Unexpected(err.to_string()))?;
        let next = match current.reserve(quantity) {
            Ok(next_level) => next_level,
            Err(domain_err) => {
                if let InventoryDomainError::InsufficientStock { .. } = &domain_err
                    && let Some(existing_id) =
                        find_reservation_by_order(&mut tx, request.order_id).await?
                {
                    tx.rollback().await.map_err(|err| db_error(&err))?;
                    return Ok(existing_id);
                }
                return Err(domain_error_to_port(domain_err));
            }
        };
        updates.push(next);
        stored_items.push(ReserveStockItem {
            sku: sku.clone(),
            quantity,
        });
    }

    for next_level in &updates {
        update_sku_stock(&mut tx, next_level).await?;
    }

    let candidate_id = ReservationId::new_v4();
    let inserted: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO stock_reservations (reservation_id, order_id, status, items, created_at)
         VALUES ($1, $2, 'RESERVED', $3, NOW())
         ON CONFLICT (order_id) DO NOTHING
         RETURNING reservation_id",
    )
    .bind(candidate_id.as_uuid())
    .bind(request.order_id.as_uuid())
    .bind(sqlx::types::Json(&stored_items))
    .fetch_optional(&mut *tx)
    .await
    .map_err(|err| db_error(&err))?;

    if let Some(id) = inserted {
        tx.commit().await.map_err(|err| db_error(&err))?;
        return Ok(ReservationId::from_uuid(id));
    }

    tx.rollback().await.map_err(|err| db_error(&err))?;
    find_reservation_by_order_on_pool(pool, request.order_id).await
}

async fn release_stock_impl(
    pool: &PgPool,
    reservation_id: ReservationId,
) -> Result<(), StockPortError> {
    let mut tx = pool.begin().await.map_err(|err| db_error(&err))?;
    let row = sqlx::query(
        "SELECT status, items
         FROM stock_reservations
         WHERE reservation_id = $1
         FOR UPDATE",
    )
    .bind(reservation_id.as_uuid())
    .fetch_optional(&mut *tx)
    .await
    .map_err(|err| db_error(&err))?;

    let Some(row) = row else {
        return Err(StockPortError::ReservationNotFound(reservation_id));
    };

    let status: String = row.try_get("status").map_err(|err| db_error(&err))?;
    if status == "RELEASED" {
        tx.commit().await.map_err(|err| db_error(&err))?;
        return Ok(());
    }
    if status != "RESERVED" {
        return Err(StockPortError::Unexpected(format!(
            "预占凭证 {reservation_id} 状态异常：{status}"
        )));
    }

    let stored_items: sqlx::types::Json<Vec<ReserveStockItem>> =
        row.try_get("items").map_err(|err| db_error(&err))?;
    let grouped = aggregate_requested_items(&stored_items.0)?;

    let mut updates = Vec::with_capacity(grouped.len());
    for (sku, release_qty) in &grouped {
        let current = lock_sku_stock(&mut tx, sku).await?;
        let quantity = Quantity::try_new(*release_qty)
            .map_err(|err| StockPortError::Unexpected(err.to_string()))?;
        let next = current.release(quantity).map_err(domain_error_to_port)?;
        updates.push(next);
    }

    for next_level in &updates {
        update_sku_stock(&mut tx, next_level).await?;
    }

    sqlx::query(
        "UPDATE stock_reservations
         SET status = 'RELEASED', released_at = NOW()
         WHERE reservation_id = $1",
    )
    .bind(reservation_id.as_uuid())
    .execute(&mut *tx)
    .await
    .map_err(|err| db_error(&err))?;

    tx.commit().await.map_err(|err| db_error(&err))?;
    Ok(())
}
