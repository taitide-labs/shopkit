CREATE TABLE IF NOT EXISTS coupons (
    code TEXT PRIMARY KEY,
    min_spend_cents BIGINT NOT NULL CHECK (min_spend_cents >= 0),
    discount_cents BIGINT NOT NULL CHECK (discount_cents > 0),
    total_quota INT NOT NULL CHECK (total_quota >= 0),
    claimed_count INT NOT NULL CHECK (claimed_count >= 0 AND claimed_count <= total_quota),
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS user_coupons (
    user_coupon_id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    code TEXT NOT NULL REFERENCES coupons (code) ON DELETE CASCADE,
    status TEXT NOT NULL,
    order_id UUID,
    claimed_at TIMESTAMPTZ NOT NULL,
    redeemed_at TIMESTAMPTZ,
    UNIQUE (user_id, code)
);
