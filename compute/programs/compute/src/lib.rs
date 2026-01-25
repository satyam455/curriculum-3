use anchor_lang::prelude::*;
// use anchor_lang::solana_program::log::sol_log_compute_units;

#[cfg(target_os = "solana")]
extern "C" {
    fn sol_log_compute_units_();
}

fn sol_log_compute_units() {
    #[cfg(target_os = "solana")]
    unsafe {
        sol_log_compute_units_();
    }
    #[cfg(not(target_os = "solana"))]
    msg!("Compute units: <mocked>");
}

declare_id!("74BJjrJyU9QY2sVAsiQgYYf8aKZS3AtzdKW1ewo1RcjW");

#[program]
pub mod compute {
    use super::*;

    pub fn initialize_item(ctx: Context<InitializeItem>, id: u64) -> Result<()> {
        let item = &mut ctx.accounts.item;
        item.id = id;
        item.value = 0;
        item.owner = ctx.accounts.authority.key();
        Ok(())
    }

    pub fn batch_process(ctx: Context<BatchProcess>, operations: Vec<Operation>) -> Result<()> {
        // Log initial compute usage
        msg!("Start batch processing");
        sol_log_compute_units();

        require!(
            operations.len() == ctx.remaining_accounts.len(),
            ErrorCode::AccountCountMismatch
        );

        for (i, op) in operations.iter().enumerate() {
            let account_info = &ctx.remaining_accounts[i];

            require!(
                account_info.owner == ctx.program_id,
                ErrorCode::InvalidOwner
            );

            // Mutability check
            require!(account_info.is_writable, ErrorCode::AccountNotWritable);

            let mut data = account_info.try_borrow_mut_data()?;
            
            
            match op.op_type {
                OpType::Add => {
                    let value_bytes = &mut data[16..24];
                    let mut value = u64::from_le_bytes(value_bytes.try_into().unwrap());
                    value = value.wrapping_add(op.amount);
                    value_bytes.copy_from_slice(&value.to_le_bytes());
                },
                OpType::Subtract => {
                     let value_bytes = &mut data[16..24];
                    let mut value = u64::from_le_bytes(value_bytes.try_into().unwrap());
                    value = value.wrapping_sub(op.amount);
                    value_bytes.copy_from_slice(&value.to_le_bytes());
                }
            }

            // Log every 5 operations to track consumption
            if i % 5 == 0 {
                msg!("Processed operation {}", i);
                sol_log_compute_units();
            }
        }

        msg!("Finished batch processing");
        sol_log_compute_units();
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeItem<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 8 + 8 + 32 // discriminator + id + value + owner
    )]
    pub item: Account<'info, Item>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BatchProcess<'info> {
    pub authority: Signer<'info>,
    // remaining_accounts will hold the Item accounts
}

#[account]
pub struct Item {
    pub id: u64,
    pub value: u64,
    pub owner: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct Operation {
    pub op_type: OpType,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum OpType {
    Add = 0,
    Subtract = 1,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Number of operations does not match number of accounts")]
    AccountCountMismatch,
    #[msg("Account owner is invalid")]
    InvalidOwner,
    #[msg("Account must be writable")]
    AccountNotWritable,
}
