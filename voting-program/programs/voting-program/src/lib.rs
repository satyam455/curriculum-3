// - Implement a voting system with proper constraint validation

use anchor_lang::prelude::*;

declare_id!("EYLpmSwivHdSJX8kHrwSrGvoXsn7jxuYThWohJwY8fbx");

const MAX_TITLE_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 256;

#[program]
pub mod voting_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum ProposalStatus {
    Active,
    Paused,
    Completed,
    Cancelled,
}


#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum UserRole {
    Voter,     // can only vote
    Proposer,  // create proposal and vote
    Moderator, // can pause/resume proposal and vote
    Admin,     // full access
}

impl UserRole {
    pub fn can_vote(&self) -> bool {
        true
    }

    pub fn can_create_proposal(&self) -> bool {
        matches!(self, UserRole::Proposer | UserRole::Moderator | UserRole::Admin) // search how its working
    }

    pub fn can_moderate(&self) -> bool {
        matches!(self, UserRole::Moderator , UserRole::Admin)
    }

    pub fn manage_role(&self) -> bool {
        matches!(self, UserRole::Admin)
    }
}

#[account]
pub struct VotinConfig {
    pub authority: Pubkey,
    pub total_proposals: u64,
    pub voting_start_time: i64,
    pub voting_end_time: i64,
    pub is_paused: bool,
    pub bump: u8,
}


#[account]
pub struct UserAccount {
    pub authority: Pubkey,
    pub role: UserRole,
    pub voting_power: u64,
    pub bump: u8,
}

#[account]
pub struct Proposal {
    pub creator: Pubkey,
    pub proposal_id: u64,
    pub title: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub status: ProposalStatus,
    pub bump: u8,
}

#[account]
pub struct VoteRecord {
    pub voter: Pubkey,
    pub proposal: Pubkey,
    pub vote_option: u8,
    pub bump: u8,
}


#[derive(Accounts)]
pub struct UpdateResource<'info> {
    pub owner: Singer<'info>,

    #[account(
        mut,
        has_one = owner @ VotingError::NotOwner
    )]
    pub resource: Account<'info, Resource>
}

#[error_code]
pub enum VotingError {
    #[msg("Not authorized")]
    Unauthorized,
    #[msg("voting not started")]
    VotingNotStarted,
    #[msg("voting ended")]
    VotingEnded,
    #[msg("already voted")]
    AlreadyVoted,
    #[msg("proposal not active")]
    ProposalNotActive,
    #[msg("Not resource owner")]
    NotOwner,
    #[msg("Locked")]
    Locked,
}

/*
voting for selected the next sol price

has_one constraint can vote
validate this for user

*/