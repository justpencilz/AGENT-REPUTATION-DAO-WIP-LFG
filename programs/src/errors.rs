use anchor_lang::prelude::*;

#[error_code]
pub enum ReputationError {
    #[msg("Agent name too long")]
    NameTooLong,
    
    #[msg("Agent already registered")]
    AgentAlreadyRegistered,
    
    #[msg("Agent not registered")]
    AgentNotRegistered,
    
    #[msg("Insufficient reputation for vouching")]
    InsufficientReputation,
    
    #[msg("Cannot vouch for yourself")]
    SelfVouchNotAllowed,
    
    #[msg("Vouch lockup period not expired")]
    LockupNotExpired,
    
    #[msg("Invalid reputation amount")]
    InvalidReputationAmount,
    
    #[msg("Task ID too long")]
    TaskIdTooLong,
    
    #[msg("Agent is inactive")]
    AgentInactive,
    
    #[msg("Decay already applied recently")]
    DecayCooldown,
    
    #[msg("Math overflow")]
    MathOverflow,
    
    #[msg("Description too long")]
    DescriptionTooLong,
    
    #[msg("Voting period has ended")]
    VotingPeriodEnded,
    
    #[msg("Voting period still active")]
    VotingPeriodActive,
    
    #[msg("Proposal already executed")]
    ProposalAlreadyExecuted,
    
    #[msg("Quorum not reached")]
    QuorumNotReached,
    
    #[msg("Proposal rejected")]
    ProposalRejected,
    
    #[msg("Invalid parameter value")]
    InvalidParameter,
    
    #[msg("Oracle not authorized")]
    OracleNotAuthorized,
    
    #[msg("Invalid proof")]
    InvalidProof,
}
