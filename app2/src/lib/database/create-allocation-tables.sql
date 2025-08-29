-- Union Airdrop Allocation Marketplace Tables
-- Run this in Supabase SQL Editor to create the tables

-- User allocations table - stores each user's total airdrop allocation
CREATE TABLE user_allocations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    total_allocation BIGINT NOT NULL DEFAULT 0,
    available_allocation BIGINT NOT NULL DEFAULT 0,
    pre_staked_amount BIGINT NOT NULL DEFAULT 0,
    shared_out_amount BIGINT NOT NULL DEFAULT 0,
    received_amount BIGINT NOT NULL DEFAULT 0,
    claimed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT check_allocation_balance CHECK (
        available_allocation + pre_staked_amount + shared_out_amount <= total_allocation + received_amount
    ),
    CONSTRAINT check_non_negative_amounts CHECK (
        total_allocation >= 0 AND 
        available_allocation >= 0 AND 
        pre_staked_amount >= 0 AND 
        shared_out_amount >= 0 AND 
        received_amount >= 0
    ),
    
    -- One allocation record per user
    UNIQUE(user_id)
);

-- Allocation shares table - tracks all sharing transactions
CREATE TABLE allocation_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    to_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    amount BIGINT NOT NULL CHECK (amount > 0),
    share_type VARCHAR(20) NOT NULL DEFAULT 'direct',
    referral_code VARCHAR(50),
    transaction_hash VARCHAR(255),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE
);

-- Pre-staking records
CREATE TABLE pre_stakes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    validator_address VARCHAR(255) NOT NULL,
    validator_name VARCHAR(255),
    amount BIGINT NOT NULL CHECK (amount > 0),
    bonus_rate DECIMAL(5,2),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    activated_at TIMESTAMP WITH TIME ZONE
);

-- Referral codes and tracking (user-generated with allocation amounts)
CREATE TABLE referral_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code VARCHAR(50) UNIQUE NOT NULL,
    allocation_amount BIGINT NOT NULL CHECK (allocation_amount > 0), -- How much they're giving away
    max_uses INTEGER NOT NULL DEFAULT 1, -- How many people can use this code
    uses_count INTEGER NOT NULL DEFAULT 0,
    total_claimed BIGINT NOT NULL DEFAULT 0, -- Total amount claimed via this code
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    expires_at TIMESTAMP WITH TIME ZONE, -- Optional expiration
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraint: can't give away more than allocated
    CONSTRAINT check_max_claimable CHECK (allocation_amount * max_uses >= total_claimed)
);


-- Social scores and gamification (materialized view)
CREATE MATERIALIZED VIEW user_social_scores AS
WITH user_stats AS (
    SELECT 
        u.id as user_id,
        u.display_name,
        -- Shares given (10 points each)
        COALESCE(shares_given.count, 0) as shares_given_count,
        COALESCE(shares_given.count * 10, 0) as shares_given_score,
        
        -- Shares received (5 points each)
        COALESCE(shares_received.count, 0) as shares_received_count,
        COALESCE(shares_received.count * 5, 0) as shares_received_score,
        
        -- Referrals used (20 points each)
        COALESCE(referrals.count, 0) as referrals_count,
        COALESCE(referrals.count * 20, 0) as referrals_score,
        
        -- Pre-staking bonus (1 point per 1000 tokens staked)
        COALESCE(FLOOR(pre_stakes.total_amount / 1000), 0) as pre_stake_bonus_score
        
    FROM users u
    
    -- Count shares given
    LEFT JOIN (
        SELECT from_user_id, COUNT(*) as count
        FROM allocation_shares 
        WHERE status = 'completed'
        GROUP BY from_user_id
    ) shares_given ON u.id = shares_given.from_user_id
    
    -- Count shares received
    LEFT JOIN (
        SELECT to_user_id, COUNT(*) as count
        FROM allocation_shares 
        WHERE status = 'completed' AND to_user_id IS NOT NULL
        GROUP BY to_user_id
    ) shares_received ON u.id = shares_received.to_user_id
    
    -- Count referrals (shares made with user's referral code)
    LEFT JOIN (
        SELECT rc.user_id, COUNT(ash.id) as count
        FROM referral_codes rc
        LEFT JOIN allocation_shares ash ON rc.code = ash.referral_code AND ash.status = 'completed'
        GROUP BY rc.user_id
    ) referrals ON u.id = referrals.user_id
    
    -- Pre-staking amounts
    LEFT JOIN (
        SELECT user_id, SUM(amount) as total_amount
        FROM pre_stakes
        WHERE status IN ('active', 'completed')
        GROUP BY user_id
    ) pre_stakes ON u.id = pre_stakes.user_id
),
ranked_users AS (
    SELECT 
        *,
        (shares_given_score + shares_received_score + referrals_score + pre_stake_bonus_score) as total_score
    FROM user_stats
)
SELECT 
    user_id,
    display_name,
    shares_given_count,
    shares_received_count,
    referrals_count,
    pre_stake_bonus_score,
    total_score,
    ROW_NUMBER() OVER (ORDER BY total_score DESC, shares_given_count DESC) as rank,
    NOW() as last_calculated_at
FROM ranked_users
ORDER BY total_score DESC;

-- Marketplace activity log for real-time feed
CREATE TABLE marketplace_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    activity_type VARCHAR(50) NOT NULL,
    amount BIGINT,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_allocation_shares_from_user ON allocation_shares(from_user_id);
CREATE INDEX idx_allocation_shares_to_user ON allocation_shares(to_user_id);
CREATE INDEX idx_allocation_shares_created_at ON allocation_shares(created_at DESC);
CREATE INDEX idx_allocation_shares_status ON allocation_shares(status);
CREATE INDEX idx_pre_stakes_user ON pre_stakes(user_id);
CREATE INDEX idx_referral_codes_user ON referral_codes(user_id);
CREATE INDEX idx_referral_codes_code ON referral_codes(code);
CREATE INDEX idx_marketplace_activities_created_at ON marketplace_activities(created_at DESC);

-- Index on materialized view for fast lookups
CREATE UNIQUE INDEX idx_user_social_scores_user_id ON user_social_scores(user_id);
CREATE INDEX idx_user_social_scores_rank ON user_social_scores(rank);
CREATE INDEX idx_user_social_scores_total_score ON user_social_scores(total_score DESC);

-- Helper function to generate random referral codes (for use in application)
CREATE OR REPLACE FUNCTION generate_referral_code()
RETURNS TEXT AS $$
DECLARE
    chars TEXT := 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';
    result TEXT := 'UNION-';
    i INTEGER;
BEGIN
    FOR i IN 1..6 LOOP
        result := result || substr(chars, floor(random() * length(chars) + 1)::integer, 1);
    END LOOP;
    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- Function to refresh social scores materialized view
CREATE OR REPLACE FUNCTION refresh_social_scores()
RETURNS VOID AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY user_social_scores;
END;
$$ LANGUAGE plpgsql;

-- To refresh social scores (run periodically or after major changes):
-- SELECT refresh_social_scores();

-- To generate a random referral code in your application:
-- SELECT generate_referral_code();
