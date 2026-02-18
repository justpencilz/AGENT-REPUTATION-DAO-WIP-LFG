use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};
use crate::state::{AgentProfile, VouchRecord, ProtocolConfig};
use crate::errors::ReputationError;

/// Weighted vouching implementation based on EigenTrust algorithm
/// Vouch impact = base_amount * (voucher_reputation / total_network_reputation)

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct VouchWeighted<'info> {
    #[account(mut)]
    pub voucher: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"agent", voucher.key().as_ref()],
        bump = voucher_profile.bump,
    )]
    pub voucher_profile: Account<'info, AgentProfile>,
    
    #[account(
        mut,
        seeds = [b"agent", vouched_for.key().as_ref()],
        bump = vouched_for_profile.bump,
    )]
    pub vouched_for_profile: Account<'info, AgentProfile>,
    
    /// CHECK: Just the pubkey of the agent being vouched for
    #[account(mut)]
    pub vouched_for: UncheckedAccount<'info>,
    
    #[account(
        init_if_needed,
        payer = voucher,
        space = WeightedVouchRecord::LEN,
        seeds = [b"weighted_vouch", voucher.key().as_ref(), vouched_for.key().as_ref()],
        bump
    )]
    pub weighted_vouch_record: Account<'info, WeightedVouchRecord>,
    
    #[account(
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, ProtocolConfig>,
    
    #[account(
        mut,
        associated_token::mint = config.reputation_mint,
        associated_token::authority = voucher,
    )]
    pub voucher_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        associated_token::mint = config.reputation_mint,
        associated_token::authority = vouch_escrow,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: PDA that owns the escrow account
    #[account(
        seeds = [b"escrow", weighted_vouch_record.key().as_ref()],
        bump,
    )]
    pub vouch_escrow: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, token::Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Extended vouch record with trust weights
#[account]
pub struct WeightedVouchRecord {
    pub voucher: Pubkey,
    pub vouched_for: Pubkey,
    pub base_amount: u64,
    pub weighted_amount: u64, // Actual reputation impact after weighting
    pub voucher_reputation_at_time: u64, // Snapshot for transparency
    pub trust_weight: u64, // Basis points (10000 = 1.0)
    pub is_positive: bool,
    pub created_at: i64,
    pub last_updated: i64,
    pub bump: u8,
}

impl WeightedVouchRecord {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 1 + 1 + 8 + 8 + 1;
}

/// Calculate trust weight based on voucher's reputation
/// Formula: trust_weight = min(voucher_reputation / min_reputation_threshold, 1.0)
/// High-reputation vouchers have more impact
pub fn calculate_trust_weight(
    voucher_reputation: u64,
    min_threshold: u64,
    max_bonus_multiplier: u64, // Basis points, e.g., 20000 = 2x max
) -> u64 {
    if voucher_reputation <= min_threshold {
        return 10000; // Base weight = 1.0
    }
    
    // Linear scaling: reputation above threshold gets bonus up to max_multiplier
    let excess = voucher_reputation.saturating_sub(min_threshold);
    let bonus = excess.saturating_mul(max_bonus_multiplier.saturating_sub(10000))
        .saturating_div(min_threshold);
    
    // Cap at max multiplier
    let weight = 10000_u64.saturating_add(bonus);
    weight.min(max_bonus_multiplier)
}

/// Calculate weighted reputation impact
pub fn calculate_weighted_impact(base_amount: u64, trust_weight: u64) -> u64 {
    base_amount.saturating_mul(trust_weight).saturating_div(10000)
}

pub fn vouch_weighted(ctx: Context<VouchWeighted>, amount: u64, is_positive: bool) -> Result<()> {
    let voucher_key = ctx.accounts.voucher.key();
    let vouched_for_key = ctx.accounts.vouched_for.key();
    
    require!(voucher_key != vouched_for_key, ReputationError::SelfVouchNotAllowed);
    require!(
        ctx.accounts.voucher_profile.reputation_score >= ctx.accounts.config.min_reputation_for_vouching,
        ReputationError::InsufficientReputation
    );
    require!(amount > 0, ReputationError::InvalidReputationAmount);
    
    let clock = Clock::get()?;
    let config = &ctx.accounts.config;
    
    // Calculate trust weight based on voucher's reputation
    let trust_weight = calculate_trust_weight(
        ctx.accounts.voucher_profile.reputation_score,
        config.min_reputation_for_vouching,
        30000, // Max 3x multiplier for whales
    );
    
    // Calculate actual weighted impact
    let weighted_amount = calculate_weighted_impact(amount, trust_weight);
    
    // Update weighted vouch record
    let vouch = &mut ctx.accounts.weighted_vouch_record;
    vouch.voucher = voucher_key;
    vouch.vouched_for = vouched_for_key;
    vouch.base_amount = amount;
    vouch.weighted_amount = weighted_amount;
    vouch.voucher_reputation_at_time = ctx.accounts.voucher_profile.reputation_score;
    vouch.trust_weight = trust_weight;
    vouch.is_positive = is_positive;
    vouch.created_at = clock.unix_timestamp;
    vouch.last_updated = clock.unix_timestamp;
    vouch.bump = ctx.bumps.weighted_vouch_record;
    
    // Apply weighted impact to target's reputation
    if is_positive {
        ctx.accounts.vouched_for_profile.reputation_score = 
            ctx.accounts.vouched_for_profile.reputation_score.saturating_add(weighted_amount);
        ctx.accounts.vouched_for_profile.positive_vouches = 
            ctx.accounts.vouched_for_profile.positive_vouches.saturating_add(1);
    } else {
        // Negative vouch: slash reputation (but don't go below 0)
        ctx.accounts.vouched_for_profile.reputation_score = 
            ctx.accounts.vouched_for_profile.reputation_score.saturating_sub(weighted_amount);
        ctx.accounts.vouched_for_profile.negative_vouches = 
            ctx.accounts.vouched_for_profile.negative_vouches.saturating_add(1);
    }
    
    // Update voucher's staked amount
    ctx.accounts.voucher_profile.staked_amount = 
        ctx.accounts.voucher_profile.staked_amount.saturating_add(amount);
    
    // Transfer tokens to escrow
    if is_positive {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.voucher_token_account.to_account_info(),
                    to: ctx.accounts.escrow_token_account.to_account_info(),
                    authority: ctx.accounts.voucher.to_account_info(),
                },
            ),
            amount,
        )?;
    }
    
    msg!("Weighted vouch: {} -> {} | Base: {}, Weight: {}bps, Impact: {}", 
        voucher_key, vouched_for_key, amount, trust_weight, weighted_amount);
    
    Ok(())
}

/// Propagate trust through the network (EigenTrust-style)
/// This aggregates reputation from the web-of-trust
#[derive(Accounts)]
pub struct PropagateTrust<'info> {
    #[account(mut)]
    pub caller: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"agent", agent.key().as_ref()],
        bump = agent_profile.bump,
    )]
    pub agent_profile: Account<'info, AgentProfile>,
    
    /// CHECK: The agent whose reputation we're updating
    pub agent: UncheckedAccount<'info>,
    
    #[account(
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, ProtocolConfig>,
}

/// Calculate propagated reputation from trust network
/// Uses PageRank-style algorithm: R_i = (1-alpha) * sum(T_ji * R_j) + alpha * E_i
pub fn propagate_trust(
    ctx: Context<PropagateTrust>,
    incoming_vouches: Vec<(Pubkey, u64, u64)>, // (voucher_pubkey, voucher_reputation, trust_weight)
) -> Result<()> {
    let profile = &mut ctx.accounts.agent_profile;
    let config = &ctx.accounts.config;
    
    // Alpha parameter for stability (typically 0.15)
    let alpha: u64 = 1500; // 0.15 in basis points
    let one_minus_alpha: u64 = 8500; // 0.85 in basis points
    
    // Calculate trust flow from incoming vouches
    let mut trust_flow: u64 = 0;
    let total_reputation: u64 = incoming_vouches.iter().map(|(_, rep, _)| rep).sum();
    
    if total_reputation > 0 {
        for (_, voucher_rep, weight) in incoming_vouches {
            // Contribution = (voucher_rep / total_rep) * weight
            let contribution = voucher_rep
                .saturating_mul(weight)
                .saturating_div(total_reputation);
            trust_flow = trust_flow.saturating_add(contribution);
        }
    }
    
    // Apply EigenTrust formula
    let propagated_reputation = trust_flow
        .saturating_mul(one_minus_alpha)
        .saturating_div(10000)
        .saturating_add(
            profile.reputation_score
                .saturating_mul(alpha)
                .saturating_div(10000)
        );
    
    // Only update if it increases reputation (prevents gaming)
    if propagated_reputation > profile.reputation_score {
        let increase = propagated_reputation.saturating_sub(profile.reputation_score);
        // Cap increase at 10% per propagation to prevent sudden spikes
        let max_increase = profile.reputation_score.saturating_div(10);
        let capped_increase = increase.min(max_increase);
        
        profile.reputation_score = profile.reputation_score.saturating_add(capped_increase);
        profile.last_activity_timestamp = Clock::get()?.unix_timestamp;
        
        msg!("Trust propagated: {} gained {} reputation from network", 
            profile.owner, capped_increase);
    }
    
    Ok(())
}
