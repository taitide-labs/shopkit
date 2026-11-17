use axum::Json;
use axum::extract::{Path, State};
use shopkit_contracts::ports::{CouponPortError, StockPortError};
use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::routing::{SliceDescriptor, SliceStatus};
use shopkit_infra::state::AppState;
use shopkit_orders_domain::errors::OrderDomainError;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

use super::command::CancelOrderResponse;
use super::handler::{self, CancelOrderError};

fn map_domain_error(err: &OrderDomainError) -> ProblemDetails {
    match err {
        OrderDomainError::AlreadyCancelled => {
            ProblemDetails::conflict("ORDER_ALREADY_CANCELLED", err.to_string())
        }
        OrderDomainError::InvalidStatusTransition(_) => {
            ProblemDetails::conflict("INVALID_ORDER_STATUS", err.to_string())
        }
        OrderDomainError::EmptyItems => {
            ProblemDetails::unprocessable("EMPTY_ORDER_ITEMS", err.to_string())
        }
        OrderDomainError::PricingOverflow => {
            ProblemDetails::unprocessable("ORDER_PRICING_OVERFLOW", err.to_string())
        }
    }
}

fn map_stock_error(err: StockPortError) -> ProblemDetails {
    match err {
        StockPortError::ReservationNotFound(_) => {
            ProblemDetails::not_found("RESERVATION_NOT_FOUND", err.to_string())
        }
        StockPortError::SkuNotFound(_) => {
            ProblemDetails::not_found("SKU_NOT_FOUND", err.to_string())
        }
        StockPortError::InsufficientStock { .. } => {
            ProblemDetails::conflict("INSUFFICIENT_STOCK", err.to_string())
        }
        StockPortError::Unexpected(msg) => ProblemDetails::internal(msg),
    }
}

fn map_coupon_error(err: CouponPortError) -> ProblemDetails {
    match err {
        CouponPortError::CouponNotOwned => {
            ProblemDetails::not_found("COUPON_NOT_FOUND", err.to_string())
        }
        CouponPortError::CouponAlreadyUsed => {
            ProblemDetails::conflict("COUPON_ALREADY_USED", err.to_string())
        }
        CouponPortError::MinimumSpendNotMet { .. } => {
            ProblemDetails::unprocessable("COUPON_MINIMUM_SPEND_NOT_MET", err.to_string())
        }
        CouponPortError::Unexpected(msg) => ProblemDetails::internal(msg),
    }
}

fn map_error(err: CancelOrderError) -> ProblemDetails {
    match err {
        CancelOrderError::NotFound(order_id) => {
            ProblemDetails::not_found("ORDER_NOT_FOUND", format!("订单 {order_id} 不存在"))
        }
        CancelOrderError::Domain(domain_err) => map_domain_error(&domain_err),
        CancelOrderError::Stock(stock_err) => map_stock_error(stock_err),
        CancelOrderError::Coupon(coupon_err) => map_coupon_error(coupon_err),
        CancelOrderError::Internal(msg) => ProblemDetails::internal(msg),
    }
}

#[utoipa::path(
    post,
    path = "/api/orders/{order_id}/cancel",
    params(
        ("order_id" = Uuid, Path, description = "订单 ID")
    ),
    responses(
        (status = 200, description = "订单取消成功并释放库存与优惠券", body = CancelOrderResponse),
        (status = 404, description = "订单不存在", body = ProblemDetails),
        (status = 409, description = "订单已取消或状态不允许取消", body = ProblemDetails),
    ),
    tag = "orders"
)]
async fn cancel_order_endpoint(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<CancelOrderResponse>, ProblemDetails> {
    handler::handle(&state, order_id)
        .await
        .map(Json)
        .map_err(map_error)
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(cancel_order_endpoint))
}

inventory::submit! {
    SliceDescriptor {
        context: "orders",
        name: "cmd_cancel_order",
        status: SliceStatus::Active,
        router_fn: router,
    }
}
