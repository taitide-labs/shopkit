use shopkit_primitives::ids::{UserCouponId, UserId};
use shopkit_primitives::simple_types::CouponCode;
use uuid::Uuid;

use super::super::command::{ClaimCouponInput, ClaimCouponResponse};
use super::super::workflow::{
    ClaimDecision, ClaimWorkflowError, CouponCampaignState, evaluate_claim,
};

fn sample_input() -> ClaimCouponInput {
    ClaimCouponInput {
        user_id: UserId::from_uuid(Uuid::new_v4()),
        code: CouponCode::try_new("SAVE20").expect("valid code"),
    }
}

fn sample_campaign(total_quota: i32, claimed_count: i32) -> CouponCampaignState {
    CouponCampaignState {
        code: CouponCode::try_new("SAVE20").expect("valid code"),
        min_spend_cents: 10_000,
        discount_cents: 2_000,
        total_quota,
        claimed_count,
    }
}

#[test]
fn evaluate_claim_succeeds_within_quota_and_increments_count() {
    let input = sample_input();
    let campaign = sample_campaign(1, 0);
    let user_coupon_id = UserCouponId::new_v4();

    let decision =
        evaluate_claim(&input, &campaign, false, user_coupon_id).expect("claim succeeds");

    assert_eq!(
        decision,
        ClaimDecision {
            user_coupon_id,
            next_claimed_count: 1,
            response: ClaimCouponResponse {
                user_coupon_id,
                user_id: input.user_id,
                code: campaign.code,
                min_spend_cents: 10_000,
                discount_cents: 2_000,
            },
        }
    );
}

#[test]
fn evaluate_claim_prioritizes_already_claimed_over_quota_exhausted() {
    let input = sample_input();
    let campaign = sample_campaign(1, 1);

    assert_eq!(
        evaluate_claim(&input, &campaign, true, UserCouponId::new_v4()),
        Err(ClaimWorkflowError::AlreadyClaimed)
    );
}

#[test]
fn evaluate_claim_rejects_exhausted_and_invalid_quota_states() {
    let input = sample_input();

    assert_eq!(
        evaluate_claim(
            &input,
            &sample_campaign(2, 2),
            false,
            UserCouponId::new_v4()
        ),
        Err(ClaimWorkflowError::QuotaExhausted)
    );
    assert_eq!(
        evaluate_claim(
            &input,
            &sample_campaign(0, 0),
            false,
            UserCouponId::new_v4()
        ),
        Err(ClaimWorkflowError::QuotaExhausted)
    );
    assert_eq!(
        evaluate_claim(
            &input,
            &sample_campaign(1, 2),
            false,
            UserCouponId::new_v4()
        ),
        Err(ClaimWorkflowError::InvalidQuotaState)
    );
    assert_eq!(
        evaluate_claim(
            &input,
            &sample_campaign(-1, 0),
            false,
            UserCouponId::new_v4()
        ),
        Err(ClaimWorkflowError::InvalidQuotaState)
    );
}
