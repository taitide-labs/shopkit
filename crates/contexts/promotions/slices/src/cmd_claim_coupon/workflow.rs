use shopkit_primitives::ids::UserCouponId;
use shopkit_primitives::simple_types::CouponCode;

use super::command::{ClaimCouponInput, ClaimCouponResponse};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CouponCampaignState {
    pub code: CouponCode,
    pub min_spend_cents: i64,
    pub discount_cents: i64,
    pub total_quota: i32,
    pub claimed_count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimDecision {
    pub user_coupon_id: UserCouponId,
    pub next_claimed_count: i32,
    pub response: ClaimCouponResponse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimWorkflowError {
    AlreadyClaimed,
    QuotaExhausted,
    InvalidQuotaState,
}

pub fn evaluate_claim(
    input: &ClaimCouponInput,
    campaign: &CouponCampaignState,
    already_claimed: bool,
    user_coupon_id: UserCouponId,
) -> Result<ClaimDecision, ClaimWorkflowError> {
    if already_claimed {
        return Err(ClaimWorkflowError::AlreadyClaimed);
    }
    if campaign.total_quota < 0
        || campaign.claimed_count < 0
        || campaign.claimed_count > campaign.total_quota
    {
        return Err(ClaimWorkflowError::InvalidQuotaState);
    }
    if campaign.claimed_count == campaign.total_quota {
        return Err(ClaimWorkflowError::QuotaExhausted);
    }
    let next_claimed_count = campaign
        .claimed_count
        .checked_add(1)
        .ok_or(ClaimWorkflowError::InvalidQuotaState)?;

    Ok(ClaimDecision {
        user_coupon_id,
        next_claimed_count,
        response: ClaimCouponResponse {
            user_coupon_id,
            user_id: input.user_id,
            code: campaign.code.clone(),
            min_spend_cents: campaign.min_spend_cents,
            discount_cents: campaign.discount_cents,
        },
    })
}
