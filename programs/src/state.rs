use anchor_lang::prelude::*;

/// Protocol configuration
#[account]
pub struct ProtocolConfig {
    pub authority: Pubkey,
    pub reputation_mint: Pubkey,
    pub min_reputation_for_vouching: u64,
    pub decay_rate_per_day: u64, // basis points (10000 = 100%)
    pub vouch_lockup_period: i64, // seconds
    pub slash_threshold: u64, // basis points for slashing
    pub max_trust_multiplier: u64, // basis points (30000 = 3x)
    pub bump: u8,
}

impl ProtocolConfig {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1;
}

/// Agent profile - PDA seeded by [agent, agent_pubkey]
#[account]
pub struct AgentProfile {
    pub owner: Pubkey,
    pub name: String, // max 50 chars
    pub reputation_score: u64,
    pub total_tasks_completed: u64,
    pub last_activity_timestamp: i64,
    pub is_active: bool,
    pub positive_vouches: u64,
    pub negative_vouches: u64,
    pub staked_amount: u64,
    pub bump: u8,
}

impl AgentProfile {
    pub const LEN: usize = 8 + 32 + (4 + 50) + 8 + 8 + 8 + 1 + 8 + 8 + 8 + 1;
}

/// Vouch record - PDA seeded by [vouch, voucher, vouched_for]
#[account]
pub struct VouchRecord {
    pub voucher: Pubkey,
    pub vouched_for: Pubkey,
    pub amount: u64,
    pub is_positive: bool, // true = vouch for, false = vouch against
    pub created_at: i64,
    pub bump: u8,
}

impl VouchRecord {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 1 + 8 + 1;
}

/// Task completion record
#[account]
pub struct TaskRecord {
    pub agent: Pubkey,
    pub task_id: String, // max 100 chars
    pub reputation_earned: u64,
    pub completed_at: i64,
    pub bump: u8,
}

impl TaskRecord {
    pub const LEN: usize = 8 + 32 + (4 + 100) + 8 + 8 + 1;
}

/// Reputation calculation helpers
pub fn calculate_decay(current_reputation: u64, days_inactive: i64, decay_rate: u64) -> u64 {
    if days_inactive <= 0 {
        return current_reputation;
    }
    
    // Apply compound decay: reputation * (1 - decay_rate)^days
    let mut remaining = current_reputation;
    for _ in 0..days_inactive {
        remaining = remaining.saturating_sub(
            remaining.saturating_mul(decay_rate).saturating_div(10000)
        );
    }
    remaining
}

pub fn calculate_vouch_bonus(positive_vouches: u64, negative_vouches: u64) -> i64 {
    let total = positive_vouches.saturating_add(negative_vouches);
    if total == 0 {
        return 0;
    }
    let ratio = (positive_vouches as i64 * 100) / (total as i64);
    // Bonus ranges from -50 to +50 based on vouch ratio
    (ratio - 50) / 2
}
