use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::state::ProtocolConfig;
use crate::errors::ReputationError;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = ProtocolConfig::LEN,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, ProtocolConfig>,
    
    /// CHECK: Verified in constraint
    pub reputation_mint: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>, config: ProtocolConfig) -> Result<()> {
    let config_account = &mut ctx.accounts.config;
    
    config_account.authority = ctx.accounts.authority.key();
    config_account.reputation_mint = ctx.accounts.reputation_mint.key();
    config_account.min_reputation_for_vouching = config.min_reputation_for_vouching;
    config_account.decay_rate_per_day = config.decay_rate_per_day;
    config_account.vouch_lockup_period = config.vouch_lockup_period;
    config_account.slash_threshold = 2000; // Default 20% slash
    config_account.max_trust_multiplier = 30000; // Default 3x max multiplier
    config_account.bump = ctx.bumps.config;
    
    msg!("Protocol initialized with decay rate: {} bps, slash threshold: {} bps", 
        config.decay_rate_per_day, config_account.slash_threshold);
    Ok(())
}
