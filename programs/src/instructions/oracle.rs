use anchor_lang::prelude::*;
use crate::state::{AgentProfile, ProtocolConfig};
use crate::errors::ReputationError;

/// Oracle Integration for automated reputation updates
/// Allows verified oracles to mint reputation for off-chain achievements

#[account]
pub struct OracleRegistry {
    pub authority: Pubkey,
    pub authorized_oracles: Vec<Pubkey>, // Max 10 oracles
    pub bump: u8,
}

impl OracleRegistry {
    pub const LEN: usize = 8 + 32 + (4 + 10 * 32) + 1;
}

#[account]
pub struct OracleAttestation {
    pub oracle: Pubkey,
    pub agent: Pubkey,
    pub attestation_type: AttestationType,
    pub metadata_hash: [u8; 32], // Hash of off-chain data
    pub reputation_amount: u64,
    pub created_at: i64,
    pub bump: u8,
}

impl OracleAttestation {
    pub const LEN: usize = 8 + 32 + 32 + 1 + 32 + 8 + 8 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum AttestationType {
    GitHubCommit,        // Code contributions
    GitHubPRMerged,      // PRs merged
    OnChainTransaction,  // High-value tx volume
    HackathonWin,        // Hackathon placement
    BugBounty,           // Security findings
    CommunityContribution, // Docs, tutorials, etc.
}

#[derive(Accounts)]
pub struct InitializeOracleRegistry<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = OracleRegistry::LEN,
        seeds = [b"oracle_registry"],
        bump
    )]
    pub oracle_registry: Account<'info, OracleRegistry>,
    
    pub system_program: Program<'info, System>,
}

pub fn initialize_oracle_registry(ctx: Context<InitializeOracleRegistry>) -> Result<()> {
    let registry = &mut ctx.accounts.oracle_registry;
    registry.authority = ctx.accounts.authority.key();
    registry.authorized_oracles = Vec::new();
    registry.bump = ctx.bumps.oracle_registry;
    
    msg!("Oracle registry initialized");
    Ok(())
}

#[derive(Accounts)]
pub struct AddOracle<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"oracle_registry"],
        bump = oracle_registry.bump,
        constraint = oracle_registry.authority == authority.key()
    )]
    pub oracle_registry: Account<'info, OracleRegistry>,
}

pub fn add_oracle(ctx: Context<AddOracle>, oracle_pubkey: Pubkey) -> Result<()> {
    let registry = &mut ctx.accounts.oracle_registry;
    
    require!(
        !registry.authorized_oracles.contains(&oracle_pubkey),
        ReputationError::OracleNotAuthorized
    );
    require!(
        registry.authorized_oracles.len() < 10,
        ReputationError::InvalidParameter
    );
    
    registry.authorized_oracles.push(oracle_pubkey);
    
    msg!("Oracle added: {}", oracle_pubkey);
    Ok(())
}

#[derive(Accounts)]
#[instruction(
    attestation_type: AttestationType,
    metadata_hash: [u8; 32],
    reputation_amount: u64
)]
pub struct SubmitAttestation<'info> {
    #[account(mut)]
    pub oracle: Signer<'info>,
    
    #[account(
        seeds = [b"oracle_registry"],
        bump = oracle_registry.bump,
    )]
    pub oracle_registry: Account<'info, OracleRegistry>,
    
    #[account(
        mut,
        seeds = [b"agent", agent.key().as_ref()],
        bump = agent_profile.bump,
    )]
    pub agent_profile: Account<'info, AgentProfile>,
    
    /// CHECK: The agent being attested for
    pub agent: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = oracle,
        space = OracleAttestation::LEN,
        seeds = [
            b"attestation",
            oracle.key().as_ref(),
            agent.key().as_ref(),
            &Clock::get()?.unix_timestamp.to_le_bytes()
        ],
        bump
    )]
    pub attestation: Account<'info, OracleAttestation>,
    
    pub system_program: Program<'info, System>,
}

pub fn submit_attestation(
    ctx: Context<SubmitAttestation>,
    attestation_type: AttestationType,
    metadata_hash: [u8; 32],
    reputation_amount: u64,
) -> Result<()> {
    let oracle = ctx.accounts.oracle.key();
    let registry = &ctx.accounts.oracle_registry;
    
    // Verify oracle is authorized
    require!(
        registry.authorized_oracles.contains(&oracle),
        ReputationError::OracleNotAuthorized
    );
    
    // Validate amount based on attestation type
    let max_amount = match attestation_type {
        AttestationType::GitHubCommit => 10,
        AttestationType::GitHubPRMerged => 50,
        AttestationType::OnChainTransaction => 100,
        AttestationType::HackathonWin => 500,
        AttestationType::BugBounty => 1000,
        AttestationType::CommunityContribution => 25,
    };
    
    require!(
        reputation_amount <= max_amount,
        ReputationError::InvalidReputationAmount
    );
    
    let clock = Clock::get()?;
    let attestation = &mut ctx.accounts.attestation;
    
    attestation.oracle = oracle;
    attestation.agent = ctx.accounts.agent.key();
    attestation.attestation_type = attestation_type.clone();
    attestation.metadata_hash = metadata_hash;
    attestation.reputation_amount = reputation_amount;
    attestation.created_at = clock.unix_timestamp;
    attestation.bump = ctx.bumps.attestation;
    
    // Apply reputation to agent
    ctx.accounts.agent_profile.reputation_score = 
        ctx.accounts.agent_profile.reputation_score.saturating_add(reputation_amount);
    ctx.accounts.agent_profile.total_tasks_completed = 
        ctx.accounts.agent_profile.total_tasks_completed.saturating_add(1);
    ctx.accounts.agent_profile.last_activity_timestamp = clock.unix_timestamp;
    
    msg!("Oracle attestation: {} earned {} reputation for {:?}", 
        attestation.agent, reputation_amount, attestation_type);
    
    Ok(())
}

/// Claim reputation from multiple attestations (batch processing)
#[derive(Accounts)]
pub struct ClaimReputation<'info> {
    #[account(mut)]
    pub agent: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"agent", agent.key().as_ref()],
        bump = agent_profile.bump,
    )]
    pub agent_profile: Account<'info, AgentProfile>,
}

pub fn claim_reputation(ctx: Context<ClaimReputation>) -> Result<()> {
    // In production: This would process pending attestations
    // and mint reputation tokens to the agent
    
    msg!("Reputation claimed for {}", ctx.accounts.agent.key());
    Ok(())
}

/// Get attestation metadata URI (for off-chain verification)
pub fn get_attestation_uri(metadata_hash: [u8; 32]) -> String {
    format!("https://api.agentreputation.io/attestations/{}", 
        bs58::encode(metadata_hash).into_string())
}

use bs58;
