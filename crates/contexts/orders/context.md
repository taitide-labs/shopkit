# 订单上下文

承担平台核心交易链路，管理订单生命周期、优惠满减计价、跨上下文补偿编排与事务性集成事件留痕。

## 术语

**订单**: 客户提交购买商品清单并成功预占库存后创建的交易聚合

- 等价：`Order`
- 避免：采购单，PurchaseOrder

**订单明细**: 记录单个商品编码、成交单价与购买数量的行项目

- 等价：`OrderLine`
- 避免：商品项，LineItem

**待支付订单**: 已锁定库存预占与优惠折扣、等待后续支付完成的核心初始状态

- 等价：`UnpaidOrder`
- 避免：草稿单，DraftOrder

**已取消订单**: 触发用户退单或异常补偿后的终态订单记录

- 等价：`CancelledOrder`
- 避免：已关闭订单，ClosedOrder

**应付金额**: 订单明细小计减去合法满减优惠后的净结算金额

- 等价：`PayableAmount`
- 避免：实收款，NetPrice

## Workflow

- `PlaceOrder`: `PlaceOrderCommand` -> `OrderPlaced`
- `CancelOrder`: `CancelOrderCommand` -> `OrderCancelled`
