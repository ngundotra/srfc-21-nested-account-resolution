use anchor_lang::prelude::*;
use anchor_lang::solana_program::address_lookup_table::instruction::{
    create_lookup_table, derive_lookup_table_address,
};

declare_id!("tY637opSt6wHYGQHMkj322aWLT8xxfWkNMidEBeRPqj");

#[derive(Accounts)]
pub struct CreateLut<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub lut: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateLutReadonly<'info> {
    /// CHECK:
    pub authority: UncheckedAccount<'info>,
    /// CHECK:
    pub payer: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CloseLut<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub lut: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct CloseLutReadonly<'info> {
    /// CHECK:
    pub authority: UncheckedAccount<'info>,
    /// CHECK:
    pub recipient: UncheckedAccount<'info>,
}

#[program]
pub mod lut_manager {
    use additional_accounts_request::AdditionalAccounts;
    use anchor_lang::solana_program::{
        address_lookup_table::instruction::close_lookup_table,
        program::{invoke, set_return_data},
        system_program,
    };

    use super::*;

    pub fn preflight_create_lut(ctx: Context<CreateLutReadonly>, slot: u64) -> Result<()> {
        let mut accounts = AdditionalAccounts::new();

        let authority = &ctx.accounts.authority;

        let (lookup_table_address, _) = derive_lookup_table_address(&authority.key, slot);
        let to_check = &[(lookup_table_address, true), (system_program::id(), false)];
        let mut last_idx = 0;
        for (idx, account) in ctx.remaining_accounts.iter().enumerate() {
            if idx >= to_check.len() {
                break;
            }
            if *account.key != to_check[idx].0 {
                msg!(
                    "Invalid account {}, was expecting: {}",
                    account.key,
                    to_check[idx].0
                );
                return Err(ProgramError::InvalidAccountData.into());
            }
            if account.is_writable != to_check[idx].1 {
                msg!(
                    "Account writability incorrect. Was expecting: {}, received: {}",
                    account.is_writable,
                    to_check[idx].1
                );
                return Err(ProgramError::InvalidAccountData.into());
            }
            last_idx = idx;
        }
        msg!("Last idx: {}", last_idx);
        for idx in last_idx..to_check.len() {
            accounts.add_account(&to_check[idx].0, to_check[idx].1)?;
        }

        set_return_data(&bytemuck::bytes_of(&accounts));
        Ok(())
    }

    pub fn create_lut(ctx: Context<CreateLut>, slot: u64) -> Result<()> {
        let authority = &ctx.accounts.authority;
        let payer = &ctx.accounts.payer;
        let lut = &ctx.accounts.lut;
        let system_program = &ctx.accounts.system_program;

        let (ix, lut_address) = create_lookup_table(authority.key(), payer.key(), slot);

        if lut.key() != lut_address {
            msg!(
                "Wrong LUT address. Expected: {}, Actual: {}",
                lut_address,
                lut.key()
            );
            return Err(ProgramError::InvalidAccountData.into());
        }

        invoke(
            &ix,
            &[
                lut.to_account_info(),
                authority.to_account_info(),
                payer.to_account_info(),
                system_program.to_account_info(),
            ],
        )?;
        Ok(())
    }

    pub fn preflight_close_lut(ctx: Context<CloseLutReadonly>) -> Result<()> {
        let mut accounts = AdditionalAccounts::new();

        let authority = &ctx.accounts.authority;
        let recipient = &ctx.accounts.recipient;

        Ok(())
    }

    pub fn close_lut(ctx: Context<CloseLut>) -> Result<()> {
        let authority = &ctx.accounts.authority;
        let recipient = &ctx.accounts.recipient;
        let lut = &ctx.accounts.lut;

        let ix = close_lookup_table(lut.key(), authority.key(), recipient.key());

        invoke(
            &ix,
            &[
                lut.to_account_info(),
                authority.to_account_info(),
                recipient.to_account_info(),
            ],
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
