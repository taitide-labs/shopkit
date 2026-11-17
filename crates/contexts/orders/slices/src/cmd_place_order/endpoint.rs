use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use shopkit_contracts::ports::{CouponPortError, StockPortError};
use shopkit_infra::problem::ProblemDetails;
use shopkit_infra::routing::{SliceDescriptor, SliceStatus};
use shopkit_infra::state::AppState;
use shopkit_orders_domain::errors::OrderDomainError;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::command::{ParsePlaceOrderError, PlaceOrderBody, PlaceOrderResponse};
use super::handler::{self, PlaceOrderError};

fn map_domain_error(err: &OrderDomainError) -> ProblemDetails {
    match err {
        OrderDomainError::EmptyItems => {
            ProblemDetails::unprocessable("EMPTY_ORDER_ITEMS", err.to_string())
        }
        OrderDomainError::PricingOverflow => {
            ProblemDetails::unprocessable("ORDER_PRICING_OVERFLOW", err.to_string())
        }
        OrderDomainError::AlreadyCancelled => {
            ProblemDetails::conflict("ORDER_ALREADY_CANCELLED", err.to_string())
        }
        OrderDomainError::InvalidStatusTransition(_) => {
            ProblemDetails::conflict("INVALID_ORDER_STATUS", err.to_string())
        }
    }
}

fn map_stock_error(err: StockPortError) -> ProblemDetails {
    match err {
        StockPortError::SkuNotFound(_) => {
            ProblemDetails::not_found("SKU_NOT_FOUND", err.to_string())
        }
        StockPortError::ReservationNotFound(_) => {
            ProblemDetails::not_found("RESERVATION_NOT_FOUND", err.to_string())
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

fn map_error(err: PlaceOrderError) -> ProblemDetails {
    match err {
        PlaceOrderError::Parse(parse_err) => match parse_err {
            ParsePlaceOrderError::Primitive(prim_err) => {
                ProblemDetails::unprocessable("INVALID_INPUT", prim_err.to_string())
            }
            ParsePlaceOrderError::Domain(domain_err) => map_domain_error(&domain_err),
        },
        PlaceOrderError::Domain(domain_err) => map_domain_error(&domain_err),
        PlaceOrderError::Stock(stock_err) => map_stock_error(stock_err),
        PlaceOrderError::Coupon(coupon_err) => map_coupon_error(coupon_err),
        PlaceOrderError::OrderAlreadyExists(order_id) => {
            ProblemDetails::conflict("ORDER_ALREADY_EXISTS", format!("订单 {order_id} 已存在"))
        }
        PlaceOrderError::Internal(msg) => ProblemDetails::internal(msg),
    }
}

#[utoipa::path(
    post,
    path = "/api/orders",
    request_body = PlaceOrderBody,
    responses(
        (status = 201, description = "下单成功", body = PlaceOrderResponse),
        (status = 404, description = "商品或优惠券不存在", body = ProblemDetails),
        (status = 409, description = "库存不足、优惠券已占用或订单号冲突", body = ProblemDetails),
        (status = 422, description = "下单参数不合法或未满足满减门槛", body = ProblemDetails),
    ),
    tag = "orders"
)]
async fn place_order_endpoint(
    State(state): State<AppState>,
    Json(body): Json<PlaceOrderBody>,
) -> Result<(StatusCode, Json<PlaceOrderResponse>), ProblemDetails> {
    handler::handle(&state, body)
        .await
        .map(|response| (StatusCode::CREATED, Json(response)))
        .map_err(map_error)
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(place_order_endpoint))
}

inventory::submit! {
    SliceDescriptor {
        context: "orders",
        name: "cmd_place_order",
        status: SliceStatus::Active,
        router_fn: router,
    }
}
