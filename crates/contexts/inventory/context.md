# 库存上下文

维护商品仓库的可用库存与订单预占账本，确保多笔订单并发扣减时绝不发生超卖并支持幂等释放。

## 术语

**库存项**: 单个商品编码对应的可用与已预占数量快照

- 等价：`StockLevel`
- 避免：商品数量，ProductCount

**可用库存**: 当前允许被新订单即时锁定的剩余可售件数

- 等价：`AvailableQuantity`
- 避免：现货数，FreeStock

**预占库存**: 已分配给未取消订单但尚未出库确认的锁定件数

- 等价：`ReservedQuantity`
- 避免：冻结数，LockedStock

**预占凭证**: 关联订单标识与扣减明细清单的幂等锁库记录

- 等价：`StockReservation`
- 避免：锁单号，StockTicket

## Workflow

- `ReserveStock`: `ReserveStockCommand` -> `StockReserved`
- `ReleaseStock`: `ReleaseStockCommand` -> `StockReleased`
