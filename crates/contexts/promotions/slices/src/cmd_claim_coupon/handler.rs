use shopkit_contracts::events::IntegrationEvent;
use shopkit_infra::state::AppState;
use shopkit_primitives::ids::UserCouponId;

use super::command::{ClaimCouponInput, ClaimCouponResponse};
use super::workflow::{ClaimWorkflowError, CouponCampaignState, evaluate_claim};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimCouponError {
    NotFound,
    AlreadyClaimed,
    QuotaExhausted,
    Internal(String),
}

#[derive(sqlx::FromRow)]
struct CouponCampaignRow {
    min_spend_cents: i64,
    discount_cents: i64,
    total_quota: i32,
    claimed_count: i32,
}

fn map_workflow_error(err: ClaimWorkflowError) -> ClaimCouponError {
    match err {
        ClaimWorkflowError::AlreadyClaimed => ClaimCouponError::AlreadyClaimed,
        ClaimWorkflowError::QuotaExhausted => ClaimCouponError::QuotaExhausted,
        ClaimWorkflowError::InvalidQuotaState => {
            ClaimCouponError::Internal("优惠券配额状态异常".to_owned())
        }
    }
}

fn map_insert_error(err: &sqlx::Error) -> ClaimCouponError {
    if let Some(db_err) = err.as_database_error()
        && db_err.code().as_deref() == Some("23505")
    {
        return ClaimCouponError::AlreadyClaimed;
    }
    ClaimCouponError::Internal(err.to_string())
}

pub async fn handle(
    state: &AppState,
    input: &ClaimCouponInput,
) -> Result<ClaimCouponResponse, ClaimCouponError> {
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|err| ClaimCouponError::Internal(err.to_string()))?;

    let row = sqlx::query_as::<_, CouponCampaignRow>(
        "SELECT min_spend_cents, discount_cents, total_quota, claimed_count
         FROM coupons
         WHERE code = $1
         FOR UPDATE",
    )
    .bind(input.code.as_str())
    .fetch_optional(&mut *tx)
    .await
    .map_err(|err| ClaimCouponError::Internal(err.to_string()))?;

    let Some(campaign_row) = row else {
        return Err(ClaimCouponError::NotFound);
    };

    let already_claimed: bool = sqlx::query_scalar(
        "SELECT EXISTS (
             SELECT 1 FROM user_coupons WHERE user_id = $1 AND code = $2
         )",
    )
    .bind(input.user_id.as_uuid())
    .bind(input.code.as_str())
    .fetch_one(&mut *tx)
    .await
    .map_err(|err| ClaimCouponError::Internal(err.to_string()))?;

    let campaign = CouponCampaignState {
        code: input.code.clone(),
        min_spend_cents: campaign_row.min_spend_cents,
        discount_cents: campaign_row.discount_cents,
        total_quota: campaign_row.total_quota,
        claimed_count: campaign_row.claimed_count,
    };
    let decision = evaluate_claim(input, &campaign, already_claimed, UserCouponId::new_v4())
        .map_err(map_workflow_error)?;

    let now = state.clock.now();

    sqlx::query(
        "UPDATE coupons
         SET claimed_count = $2
         WHERE code = $1",
    )
    .bind(input.code.as_str())
    .bind(decision.next_claimed_count)
    .execute(&mut *tx)
    .await
    .map_err(|err| ClaimCouponError::Internal(err.to_string()))?;

    sqlx::query(
        "INSERT INTO user_coupons (user_coupon_id, user_id, code, status, claimed_at)
         VALUES ($1, $2, $3, 'AVAILABLE', $4)",
    )
    .bind(decision.user_coupon_id.as_uuid())
    .bind(input.user_id.as_uuid())
    .bind(input.code.as_str())
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|err| map_insert_error(&err))?;

    tx.commit()
        .await
        .map_err(|err| ClaimCouponError::Internal(err.to_string()))?;

    state.event_bus.publish(IntegrationEvent::CouponClaimed {
        user_id: input.user_id,
        code: input.code.clone(),
        occurred_at: now,
    });

    Ok(decision.response)
}
