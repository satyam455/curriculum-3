// - Implement a voting system with proper constraint validation

use anchor_lang::prelude::*;

declare_id!("EYLpmSwivHdSJX8kHrwSrGvoXsn7jxuYThWohJwY8fbx");

const MAX_TITLE_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 256;

#[program]
pub mod voting_program {
    use super::*;

    pub fn initialize_voting_system(
        ctx: Context<InitializeVoting>,
        voting_start_time: i64,
        voting_end_time: i64,
    ) -> Result<()> {
        require!(voting_end_time > voting_start_time,)
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeVoting<'info> {  
    #[account(
        init,
        payer = owner,
        space = VotingConfig::SIZE,  
        seeds = [b"voting_config"], // we want only one config for the entire program
        bump
    )]
    pub voting_config: Account<'info, VotingConfig>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct RegisterUser<'info> { 
    #[account(                         // we need to validate the correct voting config to register users for
        mut,
        seeds = [b"voting_config"],
        bump = voting_config.bump
    )]
    pub voting_config: Account<'info, VotingConfig>,

    #[account(
        init,
        payer = owner,
        space = UserAccount::SIZE,
        seeds = [b"user", owner.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    
    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AssignRole<'info> {
    #[account(                                       
        seeds = [b"voting_config"],
        bump = voting_config.bump,
        has_one = owner @ VotingError::Unauthorized  /// used to check if the admin is accessing this or not
    )]
    pub voting_config: Account<'info, VotingConfig>,

    #[account(                                         
        seeds = [b"user", owner.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>, /// used to check if the correct entity "admin" is assiging the role

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]                                  
    pub user: Account<'info, UserAccount>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,                                        //implemented to check the proposal do not get collide with each other or to verfiy if the proposal is already created or not 
        seeds = [b"voting_config"],
        bump = voting_config.bump
    )]
    pub voting_config: Account<'info, VotingConfig>,

    #[account(
        mut,                                        // we are implementing this user_account to check if the user registered has the role to create proposal.
        seeds = [b"user", owner.key().as_ref()],
        bump = user_account.bump,
        has_one = owner @ VotingError::Unauthorized
    )]
    pub user_account: Account<'info , UserAccount>,

    #[account(                                        // we are creating proposal here with unique seed.
        init,
        payer = owner,
        space = Proposal::SIZE,
        seeds = [b"proposal", voting_config.total_proposals.to_le_bytes().as_ref()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CasteVote<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"voting_config"],
        bump = voting_config.bump
    )]
    pub voting_config: Account<'info, VotingConfig>,

    #[account(
        mut,
        seeds = [b"user", owner.key().as_ref()],
        bump = user_account.bump,
        has_one = owner @ VotingError::Unauthorized
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub proposal: Account<'info, UserAccount>,

    #[account(
        init,
        payer = owner,
        space = VoteRecord::SIZE,
        seeds = [b"vote", proposal.key().as_ref(), owner.key().as_ref()],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,

    pub system_program: Program<'info, System>,
}



// #[derive(Accounts)]
// pub struct UpdateResource<'info> {
//     pub owner: Singer<'info>,

//     #[account(
//         mut,
//         has_one = owner @ VotingError::NotOwner
//     )]
//     pub resource: Account<'info, Resource>
// }



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
    pub total_users: u64,
    pub voting_start_time: i64,
    pub voting_end_time: i64,
    pub is_paused: bool,
    pub bump: u8,
}

impl VotingConfig {
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 8 + 8 + 1 + 1  // why 8 added first??
}


#[account]
pub struct UserAccount {
    pub authority: Pubkey,
    pub role: UserRole,
    pub voting_power: u64,
    pub proposal_created: u64
    pub bump: u8,
}

impl UserAccount {
    pub const SIZE: usize = 8 + 32 + 1 + 8 + 8 + 8 + 1; // why 8 added first??
}

#[account]
pub struct Proposal {
    pub creator: Pubkey,
    pub proposal_id: u64,
    pub title: String,
    pub description: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub created_at: u64,
    pub total_proposals: u64,
    pub status: ProposalStatus,
    pub bump: u8,
}

impl Proposal {
    pub const SIZE: usize = 8 + 32 + 8 + (4 + MAX_TITLE_LENGTH) + (4 + MAX_DESCRIPTION_LENGTH) +
                            8 + 8 + 8 + 8 + 1 + 1; // why 8 added at front?
}

#[account]
pub struct VoteRecord {
    pub voter: Pubkey,
    pub proposal: Pubkey,
    pub vote_option: u8,
    pub voting_power_used: u64,
    pub bump: u8,
}

impl VoteRecord {
    pub const SIZE: usize  = 8 + 32 + 32 + 1 + 8 +  1; // why 8 addded ata front?
}


// #[account]
// pub struct Resource {
//     pub owner: Pubkey,
//     pub name: String,
//     pub value: u64,
//     pub is_locked: bool,
//     pub bump: u8,
// }

// impl Resource {
//     pub const SIZE: usize = 8 + 32 + 4 + 8 + 1 + 1; 
// }


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

Here's the complete program building on your existing code:


// - Implement a voting system with proper constraint validation

use anchor_lang::prelude::*;

declare_id!("EYLpmSwivHdSJX8kHrwSrGvoXsn7jxuYThWohJwY8fbx");

// ============================================================================
// CONSTANTS
// ============================================================================
const MAX_TITLE_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 256;

// ============================================================================
// USER ROLES - Access control system
// ============================================================================
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
        matches!(self, UserRole::Proposer | UserRole::Moderator | UserRole::Admin)
    }

    pub fn can_moderate(&self) -> bool {
        matches!(self, UserRole::Moderator | UserRole::Admin)
    }

    pub fn manage_role(&self) -> bool {
        matches!(self, UserRole::Admin)
    }
}

// ============================================================================
// PROPOSAL STATUS
// ============================================================================
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum ProposalStatus {
    Active,
    Paused,
    Completed,
    Cancelled,
}

// ============================================================================
// ERROR CODES
// ============================================================================
#[error_code]
pub enum VotingError {
    #[msg("Not authorized")]
    Unauthorized,
    #[msg("Invalid role")]
    InvalidRole,
    #[msg("voting not started")]
    VotingNotStarted,
    #[msg("voting ended")]
    VotingEnded,
    #[msg("already voted")]
    AlreadyVoted,
    #[msg("proposal not active")]
    ProposalNotActive,
    #[msg("Invalid vote option")]
    InvalidVoteOption,
    #[msg("Not resource owner")]
    NotOwner,
    #[msg("Locked")]
    Locked,
    #[msg("Title too long")]
    TitleTooLong,
    #[msg("Description too long")]
    DescriptionTooLong,
}

// ============================================================================
// ACCOUNT STRUCTURES
// ============================================================================
#[account]
pub struct VotingConfig {
    pub authority: Pubkey,
    pub total_proposals: u64,
    pub total_users: u64,
    pub voting_start_time: i64,
    pub voting_end_time: i64,
    pub is_paused: bool,
    pub bump: u8,
}

impl VotingConfig {
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 8 + 8 + 1 + 1;
}

#[account]
pub struct UserAccount {
    pub authority: Pubkey,
    pub role: UserRole,
    pub voting_power: u64,
    pub proposals_created: u64,
    pub votes_cast: u64,
    pub bump: u8,
}

impl UserAccount {
    pub const SIZE: usize = 8 + 32 + 1 + 8 + 8 + 8 + 1;
}

#[account]
pub struct Proposal {
    pub creator: Pubkey,
    pub proposal_id: u64,
    pub title: String,
    pub description: String,
    pub votes_for: u64,
    pub votes_against: u64,
    pub votes_abstain: u64,
    pub status: ProposalStatus,
    pub created_at: i64,
    pub bump: u8,
}

impl Proposal {
    pub const SIZE: usize = 8 + 32 + 8 + (4 + MAX_TITLE_LENGTH) + (4 + MAX_DESCRIPTION_LENGTH)
        + 8 + 8 + 8 + 1 + 8 + 1;
}

#[account]
pub struct VoteRecord {
    pub voter: Pubkey,
    pub proposal: Pubkey,
    pub vote_option: u8,
    pub voting_power_used: u64,
    pub voted_at: i64,
    pub bump: u8,
}

impl VoteRecord {
    pub const SIZE: usize = 8 + 32 + 32 + 1 + 8 + 8 + 1;
}

#[account]
pub struct Resource {
    pub owner: Pubkey,
    pub name: String,
    pub value: u64,
    pub is_locked: bool,
    pub bump: u8,
}

impl Resource {
    pub const SIZE: usize = 8 + 32 + (4 + 32) + 8 + 1 + 1;
}

// ============================================================================
// PROGRAM INSTRUCTIONS
// ============================================================================
#[program]
pub mod voting_program {
    use super::*;

    // Initialize voting system
    pub fn initialize_voting_system(
        ctx: Context<InitializeVotingSystem>,
        voting_start_time: i64,
        voting_end_time: i64,
    ) -> Result<()> {
        require!(voting_end_time > voting_start_time, VotingError::VotingNotStarted);

        let config = &mut ctx.accounts.voting_config;
        config.authority = ctx.accounts.authority.key();
        config.total_proposals = 0;
        config.total_users = 0;
        config.voting_start_time = voting_start_time;
        config.voting_end_time = voting_end_time;
        config.is_paused = false;
        config.bump = ctx.bumps.voting_config;

        msg!("Voting system initialized by: {}", ctx.accounts.authority.key());
        Ok(())
    }

    // Register user (default: Voter role)
    pub fn register_user(ctx: Context<RegisterUser>) -> Result<()> {
        let user = &mut ctx.accounts.user_account;
        let config = &mut ctx.accounts.voting_config;

        user.authority = ctx.accounts.authority.key();
        user.role = UserRole::Voter;
        user.voting_power = 1;
        user.proposals_created = 0;
        user.votes_cast = 0;
        user.bump = ctx.bumps.user_account;

        config.total_users += 1;

        msg!("User registered: {}", ctx.accounts.authority.key());
        Ok(())
    }

    // Assign role (Admin only)
    pub fn assign_role(ctx: Context<AssignRole>, new_role: u8) -> Result<()> {
        let admin = &ctx.accounts.admin_account;
        require!(admin.role.manage_role(), VotingError::Unauthorized);

        let target = &mut ctx.accounts.target_user;
        target.role = match new_role {
            0 => UserRole::Voter,
            1 => UserRole::Proposer,
            2 => UserRole::Moderator,
            3 => UserRole::Admin,
            _ => return Err(VotingError::InvalidRole.into()),
        };

        msg!("Role assigned to: {}", target.authority);
        Ok(())
    }

    // Update voting power (Admin only)
    pub fn update_voting_power(ctx: Context<AssignRole>, new_power: u64) -> Result<()> {
        let admin = &ctx.accounts.admin_account;
        require!(admin.role.manage_role(), VotingError::Unauthorized);

        let target = &mut ctx.accounts.target_user;
        target.voting_power = new_power;

        msg!("Voting power updated for: {}", target.authority);
        Ok(())
    }

    // Create proposal
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        title: String,
        description: String,
    ) -> Result<()> {
        require!(title.len() <= MAX_TITLE_LENGTH, VotingError::TitleTooLong);
        require!(description.len() <= MAX_DESCRIPTION_LENGTH, VotingError::DescriptionTooLong);

        let user = &mut ctx.accounts.user_account;
        let config = &mut ctx.accounts.voting_config;

        require!(user.role.can_create_proposal(), VotingError::Unauthorized);
        require!(!config.is_paused, VotingError::ProposalNotActive);

        let proposal = &mut ctx.accounts.proposal;
        proposal.creator = ctx.accounts.authority.key();
        proposal.proposal_id = config.total_proposals;
        proposal.title = title;
        proposal.description = description;
        proposal.votes_for = 0;
        proposal.votes_against = 0;
        proposal.votes_abstain = 0;
        proposal.status = ProposalStatus::Active;
        proposal.created_at = Clock::get()?.unix_timestamp;
        proposal.bump = ctx.bumps.proposal;

        config.total_proposals += 1;
        user.proposals_created += 1;

        msg!("Proposal {} created", proposal.proposal_id);
        Ok(())
    }

    // Cast vote
    pub fn cast_vote(ctx: Context<CastVote>, vote_option: u8) -> Result<()> {
        let config = &ctx.accounts.voting_config;
        let user = &mut ctx.accounts.user_account;
        let proposal = &mut ctx.accounts.proposal;
        let vote_record = &mut ctx.accounts.vote_record;
        let clock = Clock::get()?;

        // Constraint validations
        require!(clock.unix_timestamp >= config.voting_start_time, VotingError::VotingNotStarted);
        require!(clock.unix_timestamp <= config.voting_end_time, VotingError::VotingEnded);
        require!(!config.is_paused, VotingError::ProposalNotActive);
        require!(proposal.status == ProposalStatus::Active, VotingError::ProposalNotActive);
        require!(vote_option <= 2, VotingError::InvalidVoteOption);

        // Apply vote
        match vote_option {
            0 => proposal.votes_for += user.voting_power,
            1 => proposal.votes_against += user.voting_power,
            2 => proposal.votes_abstain += user.voting_power,
            _ => return Err(VotingError::InvalidVoteOption.into()),
        }

        // Record vote (PDA prevents double voting)
        vote_record.voter = ctx.accounts.authority.key();
        vote_record.proposal = proposal.key();
        vote_record.vote_option = vote_option;
        vote_record.voting_power_used = user.voting_power;
        vote_record.voted_at = clock.unix_timestamp;
        vote_record.bump = ctx.bumps.vote_record;

        user.votes_cast += 1;

        msg!("Vote cast on proposal {}", proposal.proposal_id);
        Ok(())
    }

    // Pause proposal (Moderator/Admin)
    pub fn pause_proposal(ctx: Context<ModerateProposal>) -> Result<()> {
        let user = &ctx.accounts.user_account;
        require!(user.role.can_moderate(), VotingError::Unauthorized);

        let proposal = &mut ctx.accounts.proposal;
        require!(proposal.status == ProposalStatus::Active, VotingError::ProposalNotActive);

        proposal.status = ProposalStatus::Paused;
        msg!("Proposal {} paused", proposal.proposal_id);
        Ok(())
    }

    // Resume proposal (Moderator/Admin)
    pub fn resume_proposal(ctx: Context<ModerateProposal>) -> Result<()> {
        let user = &ctx.accounts.user_account;
        require!(user.role.can_moderate(), VotingError::Unauthorized);

        let proposal = &mut ctx.accounts.proposal;
        proposal.status = ProposalStatus::Active;
        msg!("Proposal {} resumed", proposal.proposal_id);
        Ok(())
    }

    // Complete proposal (creator only)
    pub fn complete_proposal(ctx: Context<CompleteProposal>) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        proposal.status = ProposalStatus::Completed;
        msg!("Proposal {} completed", proposal.proposal_id);
        Ok(())
    }

    // Cancel proposal (Admin only)
    pub fn cancel_proposal(ctx: Context<AdminModerateProposal>) -> Result<()> {
        let user = &ctx.accounts.user_account;
        require!(user.role.manage_role(), VotingError::Unauthorized);

        let proposal = &mut ctx.accounts.proposal;
        proposal.status = ProposalStatus::Cancelled;
        msg!("Proposal {} cancelled", proposal.proposal_id);
        Ok(())
    }

    // Pause voting system (Admin only)
    pub fn pause_system(ctx: Context<AdminAction>) -> Result<()> {
        ctx.accounts.voting_config.is_paused = true;
        msg!("Voting system paused");
        Ok(())
    }

    // Resume voting system (Admin only)
    pub fn resume_system(ctx: Context<AdminAction>) -> Result<()> {
        ctx.accounts.voting_config.is_paused = false;
        msg!("Voting system resumed");
        Ok(())
    }

    // ========================================================================
    // RESOURCE MANAGEMENT - Ownership validation
    // ========================================================================

    pub fn create_resource(ctx: Context<CreateResource>, name: String) -> Result<()> {
        let resource = &mut ctx.accounts.resource;
        resource.owner = ctx.accounts.owner.key();
        resource.name = name;
        resource.value = 0;
        resource.is_locked = false;
        resource.bump = ctx.bumps.resource;

        msg!("Resource created");
        Ok(())
    }

    pub fn update_resource(ctx: Context<UpdateResource>, new_value: u64) -> Result<()> {
        let resource = &mut ctx.accounts.resource;
        require!(!resource.is_locked, VotingError::Locked);

        resource.value = new_value;
        msg!("Resource updated");
        Ok(())
    }

    pub fn lock_resource(ctx: Context<UpdateResource>) -> Result<()> {
        ctx.accounts.resource.is_locked = true;
        msg!("Resource locked");
        Ok(())
    }

    pub fn unlock_resource(ctx: Context<UpdateResource>) -> Result<()> {
        ctx.accounts.resource.is_locked = false;
        msg!("Resource unlocked");
        Ok(())
    }

    pub fn transfer_resource(ctx: Context<TransferResource>) -> Result<()> {
        let resource = &mut ctx.accounts.resource;
        require!(!resource.is_locked, VotingError::Locked);

        resource.owner = ctx.accounts.new_owner.key();
        msg!("Resource transferred");
        Ok(())
    }
}

// ============================================================================
// ACCOUNT CONTEXTS
// ============================================================================

#[derive(Accounts)]
pub struct InitializeVotingSystem<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = VotingConfig::SIZE,
        seeds = [b"voting_config"],
        bump
    )]
    pub voting_config: Account<'info, VotingConfig>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterUser<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"voting_config"],
        bump = voting_config.bump
    )]
    pub voting_config: Account<'info, VotingConfig>,

    #[account(
        init,
        payer = authority,
        space = UserAccount::SIZE,
        seeds = [b"user", authority.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AssignRole<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"voting_config"],
        bump = voting_config.bump,
        has_one = authority @ VotingError::Unauthorized
    )]
    pub voting_config: Account<'info, VotingConfig>,

    #[account(
        seeds = [b"user", authority.key().as_ref()],
        bump = admin_account.bump
    )]
    pub admin_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub target_user: Account<'info, UserAccount>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"voting_config"],
        bump = voting_config.bump
    )]
    pub voting_config: Account<'info, VotingConfig>,

    #[account(
        mut,
        seeds = [b"user", authority.key().as_ref()],
        bump = user_account.bump,
        has_one = authority @ VotingError::Unauthorized
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        init,
        payer = authority,
        space = Proposal::SIZE,
        seeds = [b"proposal", voting_config.total_proposals.to_le_bytes().as_ref()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"voting_config"],
        bump = voting_config.bump
    )]
    pub voting_config: Account<'info, VotingConfig>,

    #[account(
        mut,
        seeds = [b"user", authority.key().as_ref()],
        bump = user_account.bump,
        has_one = authority @ VotingError::Unauthorized
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        init,
        payer = authority,
        space = VoteRecord::SIZE,
        seeds = [b"vote", proposal.key().as_ref(), authority.key().as_ref()],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ModerateProposal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"user", authority.key().as_ref()],
        bump = user_account.bump,
        has_one = authority @ VotingError::Unauthorized
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
}

#[derive(Accounts)]
pub struct CompleteProposal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = creator @ VotingError::Unauthorized
    )]
    pub proposal: Account<'info, Proposal>,

    /// CHECK: Used for has_one constraint
    pub creator: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct AdminModerateProposal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"voting_config"],
        bump = voting_config.bump,
        has_one = authority @ VotingError::Unauthorized
    )]
    pub voting_config: Account<'info, VotingConfig>,

    #[account(
        seeds = [b"user", authority.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
}

#[derive(Accounts)]
pub struct AdminAction<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"voting_config"],
        bump = voting_config.bump,
        has_one = authority @ VotingError::Unauthorized
    )]
    pub voting_config: Account<'info, VotingConfig>,
}

// ============================================================================
// RESOURCE MANAGEMENT CONTEXTS
// ============================================================================

#[derive(Accounts)]
#[instruction(name: String)]
pub struct CreateResource<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = Resource::SIZE,
        seeds = [b"resource", owner.key().as_ref(), name.as_bytes()],
        bump
    )]
    pub resource: Account<'info, Resource>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateResource<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        has_one = owner @ VotingError::NotOwner
    )]
    pub resource: Account<'info, Resource>,
}

#[derive(Accounts)]
pub struct TransferResource<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        has_one = owner @ VotingError::NotOwner
    )]
    pub resource: Account<'info, Resource>,

    /// CHECK: New owner can be any pubkey
    pub new_owner: AccountInfo<'info>,
}

*/