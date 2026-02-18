use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};

mod state;
mod instructions;
mod errors;

use state::*;
use instructions::*;
use errors::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod agentreputation_dao {
    use super::*;

    /// Initialize the reputation protocol with config
    pub fn initialize(ctx: Context<Initialize>, config: ProtocolConfig) -> Result<()> {
        instructions::initialize(ctx, config)
    }

    /// Register a new agent in the reputation system
    pub fn register_agent(ctx: Context<RegisterAgent>, agent_name: String) -> Result<()> {
        instructions::register_agent(ctx, agent_name)
    }

    /// Complete a task and earn reputation
    pub fn complete_task(
        ctx: Context<CompleteTask>,
        task_id: String,
        reputation_amount: u64,
    ) -> Result<()> {
        instructions::complete_task(ctx, task_id, reputation_amount)
    }

    /// Vouch for another agent (stake tokens)
    pub fn vouch_for(ctx: Context<VouchFor>, amount: u64) -> Result<()> {
        instructions::vouch_for(ctx, amount)
    }

    /// Challenge/vouch against another agent
    pub fn vouch_against(ctx: Context<VouchAgainst>, amount: u64) -> Result<()> {
        instructions::vouch_against(ctx, amount)
    }

    /// Withdraw vouch (unstake)
    pub fn withdraw_vouch(ctx: Context<WithdrawVouch>) -> Result<()> {
        instructions::withdraw_vouch(ctx)
    }

    /// Apply decay to inactive agent
    pub fn apply_decay(ctx: Context<ApplyDecay>) -> Result<()> {
        instructions::apply_decay(ctx)
    }

    /// Get agent reputation score
    pub fn get_reputation(ctx: Context<GetReputation>) -> Result<u64> {
        instructions::get_reputation(ctx)
    }

    /// Weighted vouch with EigenTrust-style reputation weighting
    pub fn vouch_weighted(
        ctx: Context<VouchWeighted>,
        amount: u64,
        is_positive: bool,
    ) -> Result<()> {
        instructions::vouch_weighted(ctx, amount, is_positive)
    }

    /// Propagate trust through the network (EigenTrust algorithm)
    pub fn propagate_trust(
        ctx: Context<PropagateTrust>,
        incoming_vouches: Vec<(Pubkey, u64, u64)>,
    ) -> Result<()> {
        instructions::propagate_trust(ctx, incoming_vouches)
    }

    /// Create governance proposal
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        proposal_type: ProposalType,
        new_value: u64,
        description: String,
    ) -> Result<()> {
        instructions::create_proposal(ctx, proposal_type, new_value, description)
    }

    /// Vote on governance proposal
    pub fn vote_proposal(ctx: Context<VoteOnProposal>, is_for: bool) -> Result<()> {
        instructions::vote_proposal(ctx, is_for)
    }

    /// Execute passed proposal
    pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
        instructions::execute_proposal(ctx)
    }

    /// Slash malicious agent
    pub fn slash_agent(ctx: Context<SlashAgent>, evidence_hash: [u8; 32]) -> Result<()> {
        instructions::slash_agent(ctx, evidence_hash)
    }

    // Oracle integration functions
    pub fn initialize_oracle_registry(ctx: Context<InitializeOracleRegistry>) -> Result<()> {
        instructions::initialize_oracle_registry(ctx)
    }

    pub fn add_oracle(ctx: Context<AddOracle>, oracle_pubkey: Pubkey) -> Result<()> {
        instructions::add_oracle(ctx, oracle_pubkey)
    }

    pub fn submit_attestation(
        ctx: Context<SubmitAttestation>,
        attestation_type: AttestationType,
        metadata_hash: [u8; 32],
        reputation_amount: u64,
    ) -> Result<()> {
        instructions::submit_attestation(ctx, attestation_type, metadata_hash, reputation_amount)
    }

    // Reputation NFT functions
    pub fn mint_reputation_nft(
        ctx: Context<MintReputationNFT>,
        metadata_uri: String,
    ) -> Result<()> {
        instructions::mint_reputation_nft(ctx, metadata_uri)
    }

    pub fn upgrade_reputation_nft(ctx: Context<UpgradeReputationNFT>) -> Result<()> {
        instructions::upgrade_reputation_nft(ctx)
    }

    pub fn verify_reputation_nft(ctx: Context<VerifyReputationNFT>) -> Result<(ReputationLevel, u64)> {
        instructions::verify_reputation_nft(ctx)
    }

    // ZK Proof functions
    pub fn initialize_zk_registry(
        ctx: Context<InitializeZKRegistry>,
        circuit_hash: [u8; 32],
        verification_key: Vec<u8>,
    ) -> Result<()> {
        instructions::initialize_zk_registry(ctx, circuit_hash, verification_key)
    }

    pub fn submit_zk_proof(
        ctx: Context<SubmitZKProof>,
        statement: ZKStatement,
        proof: Vec<u8>,
        public_inputs: Vec<u64>,
    ) -> Result<bool> {
        instructions::submit_zk_proof(ctx, statement, proof, public_inputs)
    }

    pub fn verify_zk_proof(ctx: Context<VerifyZKProof>) -> Result<(ZKStatement, bool)> {
        instructions::verify_zk_proof(ctx)
    }
}
