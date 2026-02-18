use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, MintTo, FreezeAccount};
use crate::state::AgentProfile;

/// Soulbound Reputation NFT
/// Non-transferable NFT representing agent's trust level
/// Used for Discord gating, API access tiers, airdrop eligibility

#[account]
pub struct ReputationNFT {
    pub agent: Pubkey,
    pub level: ReputationLevel,
    pub score_at_mint: u64,
    pub minted_at: i64,
    pub metadata_uri: [u8; 100], // Fixed-size URI
    pub bump: u8,
}

impl ReputationNFT {
    pub const LEN: usize = 8 + 32 + 1 + 8 + 8 + 100 + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ReputationLevel {
    Novice,      // 0-100 rep
    Contributor, // 100-500 rep
    Builder,     // 500-1000 rep
    Guardian,    // 1000-5000 rep
    Legend,      // 5000+ rep
}

impl ReputationLevel {
    pub fn from_score(score: u64) -> Self {
        match score {
            0..=100 => ReputationLevel::Novice,
            101..=500 => ReputationLevel::Contributor,
            501..=1000 => ReputationLevel::Builder,
            1001..=5000 => ReputationLevel::Guardian,
            _ => ReputationLevel::Legend,
        }
    }

    pub fn get_benefits(&self) -> Vec<&str> {
        match self {
            ReputationLevel::Novice => vec!["Basic API access"],
            ReputationLevel::Contributor => vec!["Basic API access", "Discord role: Contributor"],
            ReputationLevel::Builder => vec!["Extended API rate limits", "Discord role: Builder", "Early feature access"],
            ReputationLevel::Guardian => vec!["Premium API access", "Discord role: Guardian", "Governance voting", "Revenue sharing"],
            ReputationLevel::Legend => vec!["Unlimited API access", "Discord role: Legend", "Full governance", "Revenue sharing", "Exclusive events"],
        }
    }
}

#[derive(Accounts)]
pub struct MintReputationNFT<'info> {
    #[account(mut)]
    pub agent: Signer<'info>,
    
    #[account(
        seeds = [b"agent", agent.key().as_ref()],
        bump = agent_profile.bump,
    )]
    pub agent_profile: Account<'info, AgentProfile>,
    
    #[account(
        init,
        payer = agent,
        space = ReputationNFT::LEN,
        seeds = [b"reputation_nft", agent.key().as_ref()],
        bump
    )]
    pub reputation_nft: Account<'info, ReputationNFT>,
    
    /// CHECK: Verified in constraint
    #[account(
        init,
        payer = agent,
        mint::decimals = 0,
        mint::authority = mint_authority,
        mint::freeze_authority = mint_authority,
    )]
    pub nft_mint: Account<'info, Mint>,
    
    #[account(
        init,
        payer = agent,
        associated_token::mint = nft_mint,
        associated_token::authority = agent,
    )]
    pub agent_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: PDA that acts as mint authority
    #[account(seeds = [b"mint_authority"], bump)]
    pub mint_authority: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn mint_reputation_nft(
    ctx: Context<MintReputationNFT>,
    metadata_uri: String,
) -> Result<()> {
    require!(metadata_uri.len() <= 100, crate::errors::ReputationError::DescriptionTooLong);
    
    let agent = ctx.accounts.agent.key();
    let agent_profile = &ctx.accounts.agent_profile;
    let clock = Clock::get()?;
    
    // Determine level based on current reputation
    let level = ReputationLevel::from_score(agent_profile.reputation_score);
    
    // Convert String to fixed-size array
    let mut uri_bytes = [0u8; 100];
    let uri_slice = metadata_uri.as_bytes();
    uri_bytes[..uri_slice.len()].copy_from_slice(uri_slice);
    
    // Create NFT account
    let nft = &mut ctx.accounts.reputation_nft;
    nft.agent = agent;
    nft.level = level.clone();
    nft.score_at_mint = agent_profile.reputation_score;
    nft.minted_at = clock.unix_timestamp;
    nft.metadata_uri = uri_bytes;
    nft.bump = ctx.bumps.reputation_nft;
    
    // Mint the NFT (soulbound - non-transferable)
    let seeds = &[b"mint_authority", &[ctx.bumps.mint_authority]];
    let signer = &[&seeds[..]];
    
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.agent_token_account.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer,
        ),
        1, // NFT = 1 token
    )?;
    
    // Freeze the account to make it soulbound (non-transferable)
    token::freeze_account(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::FreezeAccount {
                account: ctx.accounts.agent_token_account.to_account_info(),
                mint: ctx.accounts.nft_mint.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer,
        ),
    )?;
    
    msg!("Soulbound Reputation NFT minted for {} at level {:?}", 
        agent, level);
    msg!("Benefits: {:?}", level.get_benefits());
    
    Ok(())
}

#[derive(Accounts)]
pub struct UpgradeReputationNFT<'info> {
    #[account(mut)]
    pub agent: Signer<'info>,
    
    #[account(
        seeds = [b"agent", agent.key().as_ref()],
        bump = agent_profile.bump,
    )]
    pub agent_profile: Account<'info, AgentProfile>,
    
    #[account(
        mut,
        seeds = [b"reputation_nft", agent.key().as_ref()],
        bump = reputation_nft.bump,
        constraint = reputation_nft.agent == agent.key()
    )]
    pub reputation_nft: Account<'info, ReputationNFT>,
}

pub fn upgrade_reputation_nft(ctx: Context<UpgradeReputationNFT>) -> Result<()> {
    let agent_profile = &ctx.accounts.agent_profile;
    let nft = &mut ctx.accounts.reputation_nft;
    
    let new_level = ReputationLevel::from_score(agent_profile.reputation_score);
    
    require!(
        new_level != nft.level,
        crate::errors::ReputationError::InvalidParameter
    );
    
    let old_level = nft.level.clone();
    nft.level = new_level.clone();
    nft.score_at_mint = agent_profile.reputation_score; // Update score snapshot
    
    msg!("Reputation NFT upgraded from {:?} to {:?}", old_level, new_level);
    msg!("New benefits: {:?}", new_level.get_benefits());
    
    Ok(())
}

#[derive(Accounts)]
pub struct VerifyReputationNFT<'info> {
    pub verifier: Signer<'info>,
    
    #[account(
        seeds = [b"reputation_nft", agent.key().as_ref()],
        bump = reputation_nft.bump,
    )]
    pub reputation_nft: Account<'info, ReputationNFT>,
    
    /// CHECK: The agent being verified
    pub agent: UncheckedAccount<'info>,
}

pub fn verify_reputation_nft(ctx: Context<VerifyReputationNFT>) -> Result<(ReputationLevel, u64)> {
    let nft = &ctx.accounts.reputation_nft;
    
    msg!("Verification: {} has {:?} level (score at mint: {})", 
        nft.agent, nft.level, nft.score_at_mint);
    
    Ok((nft.level.clone(), nft.score_at_mint))
}
