use anchor_lang::prelude::*;

declare_id!("EYLpmSwivHdSJX8kHrwSrGvoXsn7jxuYThWohJwY8fbx");

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
