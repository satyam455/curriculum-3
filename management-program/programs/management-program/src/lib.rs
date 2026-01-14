use anchor_lang::prelude::*;

declare_id!("EwWTXinwaED6J5rKC5yVH5AkfAzqZmNyvngGRGTKNT6g");

#[program]
pub mod management_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
