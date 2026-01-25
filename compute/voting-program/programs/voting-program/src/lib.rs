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
        require!(voting_end_time > voting_start_time, VotingError::VotingNotStarted);

        let config = &mut ctx.accounts.voting_config;
        config.owner = ctx.accounts.owner.key();
        config.total_proposals = 0;
        config.total_users = 0;
        config.voting_start_time = voting_start_time;
        config.voting_end_time = voting_end_time;
        config.is_paused = false;
        config.bump = ctx.bumps.voting_config;
     
        msg!("voting system initialized by {:?}", ctx.accounts.owner.key());
        Ok(())
    }

    pub fn register_user(ctx: Context<RegisterUser>) -> Result<()> {
        let user = &mut ctx.accounts.user_account;
        let config = &mut ctx.accounts.voting_config;

        user.owner = ctx.accounts.owner.key();
        user.role = UserRole::Voter;
        user.voting_power = 1;
        user.proposal_created = 0;
        user.votes_cast = 0;
        user.bump = ctx.bumps.user_account;

        config.total_users += 1;

        msg!("user registered: {}", ctx.accounts.owner.key());
        Ok(())

    }

    pub fn assign_role(ctx: Context<AssignRole>, new_role: u8) -> Result<()> {
        let admin = &mut ctx.accounts.admin_account;

        require!(admin.role.manage_role(), VotingError::Unauthorized);

        let target = &mut ctx.accounts.user;
        target.role = match new_role {
         0 => UserRole::Voter,
         1 => UserRole::Proposer,
         2 => UserRole::Moderator,
         3 => UserRole::Admin,
         _ => return Err(VotingError::InvalidRole.into()),
        };

        msg!("role assigned to {}", target.owner);
        Ok(())
    }

    pub fn update_voting_power(ctx: Context<AssignRole>, new_power: u64) -> Result<()> {
        let admin = &mut ctx.accounts.admin_account;
        require!(admin.role.manage_role(), VotingError::Unauthorized);

        let target = &mut ctx.accounts.user;
        target.voting_power = new_power;

        msg!("voting power updated for : {}", target.owner);
        Ok(())
    }

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        title: String,
        description: String
    ) -> Result<()> {
        require!(title.len() <= MAX_TITLE_LENGTH, VotingError::TitleTooLong);
        require!(description.len() <= MAX_DESCRIPTION_LENGTH, VotingError::DescriptionTooLong);

        let user = &mut ctx.accounts.user_account;
        let config = &mut ctx.accounts.voting_config;

        require!(user.role.can_create_proposal(), VotingError::Unauthorized);
        require!(!config.is_paused, VotingError::ProposalNotActive);

        let proposal = &mut ctx.accounts.proposal;
        proposal.creator = ctx.accounts.owner.key();
        proposal.proposal_id = config.total_proposals;
        proposal.title = title;
        proposal.description = description;
        proposal.votes_for = 0;
        proposal.votes_against = 0;
        proposal.status = ProposalStatus::Active;
        proposal.created_at = Clock::get()?.unix_timestamp;
        proposal.bump = ctx.bumps.proposal;

        config.total_proposals += 1;
        user.proposal_created += 1;
        
        msg!("proposal {} created", proposal.proposal_id);
        Ok(())
    }


    pub fn cast_vote(ctx: Context<CastVote>, vote_option: u8) -> Result<()> {

        let config = &ctx.accounts.voting_config;
        let user = &mut ctx.accounts.user_account;
        let proposal = &mut ctx.accounts.proposal;
        let vote_record = &mut ctx.accounts.vote_record;
        let clock = Clock::get();

        require!(clock.unix_timestamp >= config.voting_start_time, VotingError::VotingNotStarted);
        require!(clock.unix_timestamp <= config.voting_end_time, VotingError::VotingEnded);
        require!(!config.is_paused, VotingError::ProposalNotActive);
        require!(proposal.status == ProposalStatus::Active, VotingError::ProposalNotActive);
        require!(vote_option <= 2);

        match vote_option {
            0 => proposal.votes_for += user.voting_power,
            1 => proposal.votes_against += user.voting_power,
            2 => proposal.votes_abstain += user.voting_power,
            _ => return Err(VotingError::InvalidVoteOption.into()),
        }

        vote_record.voter = ctx.accounts.owner.key();
        vote_record.proposal = proposal.key();
        vote_record.vote_opion = vote_option;
        vote_record.voting_power_used = user.voting_power;
        vote_record.vote_at = clock.unix_timestamp;
        vote_record.bump = ctx.bumps.vote_record;

        user.votes_cast += 1;

        msg!("vote cast on proposal {}", proposal.proposal_id);
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
        has_one = owner @ VotingError::Unauthorized  // used to check if the admin is accessing this or not
    )]
    pub voting_config: Account<'info, VotingConfig>,

    #[account(                                         
        seeds = [b"user", owner.key().as_ref()],
        bump = admin_account.bump
    )]
    pub admin_account: Account<'info, UserAccount>, // used to check if the correct entity "admin" is assiging the role

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



