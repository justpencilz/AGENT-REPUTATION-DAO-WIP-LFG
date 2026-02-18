use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, TokenAccount};
use crate::state::{AgentProfile, TaskRecord, ProtocolConfig};
use crate::errors::ReputationError;

#[derive(Accounts)]
pub struct CompleteTask<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"agent", authority.key().as_ref()],
        bump = agent_profile.bump,
    )]
    pub agent_profile: Account<'info, AgentProfile>,
    
    #[account(
        init,
        payer = authority,
        space = TaskRecord::LEN,
        seeds = [b"task", &task_id[..task_id_len as usize], authority.key().as_ref()],
        bump
    )]
    pub task_record: Account<'info, TaskRecord>,
    
    #[account(
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, ProtocolConfig>,
    
    /// CHECK: This is the token mint authority PDA
    #[account(
        seeds = [b"mint_authority"],
        bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,
    
    #[account(
        mut,
        address = config.reputation_mint
    )]
    pub reputation_mint: Account<'info, token::Mint>,
    
    #[account(
        mut,
        associated_token::mint = reputation_mint,
        associated_token::authority = authority,
    )]
    pub agent_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, token::Token>,
    pub system_program: Program<'info, System>,
}

pub fn complete_task(
    ctx: Context<CompleteTask>,
    task_id: [u8; 100],
    task_id_len: u8,
    reputation_amount: u64,
) -> Result<()> {
    require!(task_id_len <= 100, ReputationError::TaskIdTooLong);
    require!(reputation_amount > 0, ReputationError::InvalidReputationAmount);
    
    let profile = &mut ctx.accounts.agent_profile;
    let clock = Clock::get()?;
    
    require!(profile.is_active, ReputationError::AgentInactive);
    
    // Update profile
    profile.reputation_score = profile.reputation_score.saturating_add(reputation_amount);
    profile.total_tasks_completed = profile.total_tasks_completed.saturating_add(1);
    profile.last_activity_timestamp = clock.unix_timestamp;
    
    // Create task record
    let task = &mut ctx.accounts.task_record;
    task.agent = ctx.accounts.authority.key();
    task.task_id = task_id;
    task.reputation_earned = reputation_amount;
    task.completed_at = clock.unix_timestamp;
    task.bump = ctx.bumps.task_record;
    
    // Mint reputation tokens to agent
    let seeds = &[b"mint_authority", &[ctx.bumps.mint_authority]];
    let signer = &[&seeds[..]];
    
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.reputation_mint.to_account_info(),
                to: ctx.accounts.agent_token_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer,
        ),
        reputation_amount,
    )?;
    
    msg!("Task completed: {} | Reputation earned: {}", task_id, reputation_amount);
    Ok(())
}
