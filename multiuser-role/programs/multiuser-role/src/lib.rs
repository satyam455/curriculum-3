use anchor_lang::prelude::*;

declare_id!("2vb4qKVqxqLdpP31aJzbdqztYzzseBW3YRp3WRQmipZy");

#[program]
pub mod multiuser_role {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
