# 优惠券上下文

负责促销批次配额分发、用户领券防重控制以及下单核销与退单回退。

## 术语

**优惠券批次**: 定义固定面额减免与满减门槛及发行总限额的促销模板

- 等价：`CouponCampaign`
- 避免：活动券，PromoTemplate

**用户券**: 归属于特定客户账号且在单个订单中至多使用一次的电子凭证

- 等价：`UserCoupon`
- 避免：卡券，Voucher

**满减规则**: 约束最低消费金额与立减额度的不可变值规则

- 等价：`DiscountRule`
- 避免：折扣公式，PromoRule

## Workflow

- `ClaimCoupon`: `ClaimCouponCommand` -> `CouponClaimed`
