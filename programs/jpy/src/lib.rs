use anchor_lang::prelude::*;

declare_id!("Ck3EtXJLFE2dKYexEnCGc6psdUmRZLgFDbo9nPVhmezJ");

#[program]
pub mod jpy {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
