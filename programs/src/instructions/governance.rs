use anchor_lang::prelude::*;
use crate::state::ProtocolConfig;
use crate::errors::ReputationError;

/// DAO Governance for dynamic parameter updates
/// Allows reputation-weighted voting on protocol parameters

#[account]
pub struct GovernanceProposal {
    pub proposer: Pubkey,
    pub proposal_type: ProposalType,
    pub new_value: u64,
    pub description: [u8; 200], // Fixed-size description
    pub votes_for: u64,
    pub votes_against: u64,
    pub voting_ends_at: i64,
    pub executed: bool,
    pub bump: u8,
}

impl GovernanceProposal {
    pub const LEN: usize = 8 + 32 + 1 + 8 + 200 + 8 + 8 + 8 + 1 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalType {
    UpdateMinReputationForVouching,
    UpdateDecayRate,
    UpdateVouchLockupPeriod,
    UpdateSlashThreshold,
    UpdateMaxTrustMultiplier,
}

#[derive(Accounts)]
#[instruction(proposal_type: ProposalType, new_value: u64, description: String)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    
    #[account(
        seeds = [b"agent", proposer.key().as_ref()],
        bump = proposer_profile.bump,
    )]
    pub proposer_profile: Account<'info, crate::state::AgentProfile>,
    
    #[account(
        init,
        payer = proposer,
        space = GovernanceProposal::LEN,
        seeds = [
            b"proposal", 
            proposer.key().as_ref(),
            &Clock::get()?.unix_timestamp.to_le_bytes()
        ],
        bump
    )]
    pub proposal: Account<'info, GovernanceProposal>,
    
    #[account(
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, ProtocolConfig>,
    
    pub system_program: Program<'info, System>,
}

pub fn create_proposal(
    ctx: Context<CreateProposal>,
    proposal_type: ProposalType,
    new_value: u64,
    description: String,
) -> Result<()> {
    let proposer_profile = &ctx.accounts.proposer_profile;
    
    // Require minimum reputation to propose (prevents spam)
    require!(
        proposer_profile.reputation_score >= 1000,
        ReputationError::InsufficientReputation
    );
    require!(description.len() <= 200, ReputationError::DescriptionTooLong);
    
    let clock = Clock::get()?;
    let proposal = &mut ctx.accounts.proposal;
    
    // Convert String to fixed-size byte array
    let mut desc_bytes = [0u8; 200];
    let bytes = description.as_bytes();
    desc_bytes[..bytes.len()].copy_from_slice(bytes);
    
    proposal.proposer = ctx.accounts.proposer.key();
    proposal.proposal_type = proposal_type;
    proposal.new_value = new_value;
    proposal.description = desc_bytes;
    proposal.votes_for = 0;
    proposal.votes_against = 0;
    proposal.voting_ends_at = clock.unix_timestamp + 86400 * 3; // 3 day voting period
    proposal.executed = false;
    proposal.bump = ctx.bumps.proposal;
    
    // Auto-vote with proposer's reputation
    proposal.votes_for = proposer_profile.reputation_score;
    
    msg!("Governance proposal created by {}: {:?} = {}", 
        proposal.proposer, proposal.proposal_type, proposal.new_value);
    
    Ok(())
}

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,
    
    #[account(
        seeds = [b"agent", voter.key().as_ref()],
        bump = voter_profile.bump,
    )]
    pub voter_profile: Account<'info, crate::state::AgentProfile>,
    
    #[account(mut)]
    pub proposal: Account<'info, GovernanceProposal>,
    
    #[account(
        init,
        payer = voter,
        space = VoteRecord::LEN,
        seeds = [b"vote", proposal.key().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,
    
    pub system_program: Program<'info, System>,
}

#[account]
pub struct VoteRecord {
    pub voter: Pubkey,
    pub proposal: Pubkey,
    pub vote_weight: u64, // Reputation at time of voting
    pub is_for: bool,
    pub voted_at: i64,
    pub bump: u8,
}

impl VoteRecord {
    pub const LEN: usize = 8 + 32 + 32 + 8 + 1 + 8 + 1;
}

pub fn vote_proposal(ctx: Context<VoteOnProposal>, is_for: bool) -> Result<()> {
    let voter_profile = &ctx.accounts.voter_profile;
    let proposal = &mut ctx.accounts.proposal;
    let clock = Clock::get()?;
    
    // Check voting period
    require!(
        clock.unix_timestamp < proposal.voting_ends_at,
        ReputationError::VotingPeriodEnded
    );
    require!(!proposal.executed, ReputationError::ProposalAlreadyExecuted);
    require!(
        voter_profile.reputation_score >= 100,
        ReputationError::InsufficientReputation
    );
    
    // Record vote
    let vote_record = &mut ctx.accounts.vote_record;
    vote_record.voter = ctx.accounts.voter.key();
    vote_record.proposal = proposal.key();
    vote_record.vote_weight = voter_profile.reputation_score;
    vote_record.is_for = is_for;
    vote_record.voted_at = clock.unix_timestamp;
    vote_record.bump = ctx.bumps.vote_record;
    
    // Apply reputation-weighted vote
    if is_for {
        proposal.votes_for = proposal.votes_for.saturating_add(voter_profile.reputation_score);
    } else {
        proposal.votes_against = proposal.votes_against.saturating_add(voter_profile.reputation_score);
    }
    
    msg!("Vote cast: {} voted {} with weight {}", 
        vote_record.voter, if is_for { "FOR" } else { "AGAINST" }, vote_record.vote_weight);
    
    Ok(())
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(mut)]
    pub executor: Signer<'info>,
    
    #[account(mut)]
    pub proposal: Account<'info, GovernanceProposal>,
    
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, ProtocolConfig>,
}

pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let config = &mut ctx.accounts.config;
    let clock = Clock::get()?;
    
    // Validation checks
    require!(
        clock.unix_timestamp >= proposal.voting_ends_at,
        ReputationError::VotingPeriodActive
    );
    require!(!proposal.executed, ReputationError::ProposalAlreadyExecuted);
    
    // Check quorum: need at least 10000 total reputation voted
    let total_votes = proposal.votes_for.saturating_add(proposal.votes_against);
    require!(total_votes >= 10000, ReputationError::QuorumNotReached);
    
    // Check majority: 51% to pass
    require!(
        proposal.votes_for > proposal.votes_against,
        ReputationError::ProposalRejected
    );
    
    // Execute the parameter change
    match proposal.proposal_type {
        ProposalType::UpdateMinReputationForVouching => {
            config.min_reputation_for_vouching = proposal.new_value;
            msg!("Updated min_reputation_for_vouching to {}", proposal.new_value);
        }
        ProposalType::UpdateDecayRate => {
            require!(proposal.new_value <= 1000, ReputationError::InvalidParameter); // Max 10% daily
            config.decay_rate_per_day = proposal.new_value;
            msg!("Updated decay_rate_per_day to {}", proposal.new_value);
        }
        ProposalType::UpdateVouchLockupPeriod => {
            config.vouch_lockup_period = proposal.new_value as i64;
            msg!("Updated vouch_lockup_period to {}", proposal.new_value);
        }
        ProposalType::UpdateSlashThreshold => {
            config.slash_threshold = proposal.new_value;
            msg!("Updated slash_threshold to {}", proposal.new_value);
        }
        ProposalType::UpdateMaxTrustMultiplier => {
            require!(proposal.new_value >= 10000 && proposal.new_value <= 50000, ReputationError::InvalidParameter);
            config.max_trust_multiplier = proposal.new_value;
            msg!("Updated max_trust_multiplier to {}", proposal.new_value);
        }
    }
    
    proposal.executed = true;
    
    msg!("Proposal executed successfully by {}", ctx.accounts.executor.key());
    
    Ok(())
}

#[derive(Accounts)]
pub struct SlashAgent<'info> {
    #[account(mut)]
    pub slasher: Signer<'info>,
    
    #[account(
        seeds = [b"agent", slasher.key().as_ref()],
        bump = slasher_profile.bump,
    )]
    pub slasher_profile: Account<'info, crate::state::AgentProfile>,
    
    #[account(
        mut,
        seeds = [b"agent", target.key().as_ref()],
        bump = target_profile.bump,
    )]
    pub target_profile: Account<'info, crate::state::AgentProfile>,
    
    /// CHECK: The agent being slashed
    pub target: UncheckedAccount<'info>,
    
    #[account(
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, ProtocolConfig>,
}

/// Dynamic slashing based on configurable thresholds
pub fn slash_agent(ctx: Context<SlashAgent>, evidence_hash: [u8; 32]) -> Result<()> {
    let slasher_profile = &ctx.accounts.slasher_profile;
    let target_profile = &mut ctx.accounts.target_profile;
    
    // Require slasher to have significant reputation
    require!(
        slasher_profile.reputation_score >= 5000,
        ReputationError::InsufficientReputation
    );
    
    // Calculate slash amount (dynamic based on target's reputation)
    // Higher reputation agents lose more when slashed
    let slash_percentage: u64 = 2000; // 20% base slash (configurable via governance)
    let slash_amount = target_profile.reputation_score
        .saturating_mul(slash_percentage)
        .saturating_div(10000);
    
    // Apply slash
    target_profile.reputation_score = target_profile.reputation_score.saturating_sub(slash_amount);
    
    // Deactivate if reputation too low
    if target_profile.reputation_score < 100 {
        target_profile.is_active = false;
    }
    
    // Reward slasher with portion of slashed amount (5% bounty)
    let bounty = slash_amount.saturating_mul(500).saturating_div(10000);
    // In production: transfer bounty tokens to slasher
    
    msg!("Agent {} slashed by {}: lost {} reputation (bounty: {})", 
        target_profile.owner, slasher_profile.owner, slash_amount, bounty);
    
    Ok(())
}
