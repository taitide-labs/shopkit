use std::collections::{BTreeMap, HashMap};

use parking_lot::Mutex;
use shopkit_contracts::ports::{
    BoxFuture, CouponPortError, CouponRedemptionPort, RedeemCouponRequest, ReserveStockRequest,
    StockPortError, StockReservationPort,
};
use shopkit_primitives::ids::{OrderId, ReservationId, UserId};
use shopkit_primitives::simple_types::{CouponCode, DiscountRule, Money};

#[derive(Debug, Clone, PartialEq, Eq)]
struct FakeReservation {
    items: BTreeMap<String, u32>,
    released: bool,
}

#[derive(Debug, Default)]
pub struct FakeStockReservationPort {
    stock: Mutex<HashMap<String, u32>>,
    reservations: Mutex<HashMap<ReservationId, FakeReservation>>,
    order_index: Mutex<HashMap<OrderId, ReservationId>>,
}

impl FakeStockReservationPort {
    #[must_use]
    pub fn with_stock<const N: usize>(entries: [(&str, u32); N]) -> Self {
        let mut stock = HashMap::new();
        for (sku, qty) in entries {
            stock.insert(sku.to_owned(), qty);
        }
        Self {
            stock: Mutex::new(stock),
            reservations: Mutex::new(HashMap::new()),
            order_index: Mutex::new(HashMap::new()),
        }
    }

    #[must_use]
    pub fn available(&self, sku: &str) -> u32 {
        self.stock.lock().get(sku).copied().unwrap_or(0)
    }
}

impl StockReservationPort for FakeStockReservationPort {
    fn reserve_stock<'a>(
        &'a self,
        request: &'a ReserveStockRequest,
    ) -> BoxFuture<'a, Result<ReservationId, StockPortError>> {
        Box::pin(async move {
            let mut order_index = self.order_index.lock();
            if let Some(existing) = order_index.get(&request.order_id).copied() {
                return Ok(existing);
            }

            let mut stock = self.stock.lock();
            let mut needed: BTreeMap<String, u32> = BTreeMap::new();
            for item in &request.items {
                let entry = needed.entry(item.sku.as_str().to_owned()).or_insert(0);
                *entry = entry.saturating_add(item.quantity.get());
            }

            for (sku, requested) in &needed {
                let Some(available) = stock.get(sku).copied() else {
                    return Err(StockPortError::SkuNotFound(sku.clone()));
                };
                if available < *requested {
                    return Err(StockPortError::InsufficientStock {
                        sku: sku.clone(),
                        requested: *requested,
                        available,
                    });
                }
            }

            for (sku, requested) in &needed {
                if let Some(slot) = stock.get_mut(sku) {
                    *slot = slot.saturating_sub(*requested);
                }
            }

            let reservation_id = ReservationId::new_v4();
            self.reservations.lock().insert(
                reservation_id,
                FakeReservation {
                    items: needed,
                    released: false,
                },
            );
            order_index.insert(request.order_id, reservation_id);
            Ok(reservation_id)
        })
    }

    fn release_stock(
        &self,
        reservation_id: ReservationId,
    ) -> BoxFuture<'_, Result<(), StockPortError>> {
        Box::pin(async move {
            let mut reservations = self.reservations.lock();
            let Some(record) = reservations.get_mut(&reservation_id) else {
                return Err(StockPortError::ReservationNotFound(reservation_id));
            };
            if record.released {
                return Ok(());
            }
            let mut stock = self.stock.lock();
            for (sku, qty) in &record.items {
                let entry = stock.entry(sku.clone()).or_insert(0);
                *entry = entry.saturating_add(*qty);
            }
            record.released = true;
            Ok(())
        })
    }
}

#[derive(Debug, Clone)]
struct FakeCouponRecord {
    rule: DiscountRule,
    used_by_order: Option<OrderId>,
}

#[derive(Debug, Default)]
pub struct FakeCouponRedemptionPort {
    owned: Mutex<HashMap<(UserId, CouponCode), FakeCouponRecord>>,
}

impl FakeCouponRedemptionPort {
    pub fn grant_coupon(&self, user_id: UserId, code: CouponCode, rule: DiscountRule) {
        self.owned.lock().insert(
            (user_id, code),
            FakeCouponRecord {
                rule,
                used_by_order: None,
            },
        );
    }

    #[must_use]
    pub fn is_available(&self, user_id: UserId, code: &CouponCode) -> bool {
        self.owned
            .lock()
            .get(&(user_id, code.clone()))
            .is_some_and(|record| record.used_by_order.is_none())
    }
}

impl CouponRedemptionPort for FakeCouponRedemptionPort {
    fn redeem_coupon<'a>(
        &'a self,
        request: &'a RedeemCouponRequest,
    ) -> BoxFuture<'a, Result<DiscountRule, CouponPortError>> {
        Box::pin(async move {
            let mut guard = self.owned.lock();
            let Some(record) = guard.get_mut(&(request.user_id, request.code.clone())) else {
                return Err(CouponPortError::CouponNotOwned);
            };
            if let Some(existing_order) = record.used_by_order
                && existing_order != request.order_id
            {
                return Err(CouponPortError::CouponAlreadyUsed);
            }
            if request.subtotal < record.rule.min_spend {
                return Err(CouponPortError::MinimumSpendNotMet {
                    min_spend_cents: record.rule.min_spend.as_cents(),
                });
            }
            record.used_by_order = Some(request.order_id);
            Ok(
                DiscountRule::try_new(record.rule.min_spend, record.rule.discount).unwrap_or(
                    DiscountRule {
                        min_spend: Money::ZERO,
                        discount: Money::ZERO,
                    },
                ),
            )
        })
    }

    fn release_coupon(
        &self,
        user_id: UserId,
        order_id: OrderId,
    ) -> BoxFuture<'_, Result<(), CouponPortError>> {
        Box::pin(async move {
            let mut guard = self.owned.lock();
            for ((uid, _code), record) in guard.iter_mut() {
                if *uid == user_id && record.used_by_order == Some(order_id) {
                    record.used_by_order = None;
                }
            }
            Ok(())
        })
    }
}
