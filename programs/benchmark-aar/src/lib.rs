use additional_accounts_request::{IAccountMeta, PreflightPayload};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::set_return_data;

declare_id!("BjchWSbz7LBQFQpH64cJTcL8qPiGQ8UM4n6Mmd67huS7");

fn create_accounts(num_accounts: u32) -> Vec<IAccountMeta> {
    let mut accounts: Vec<IAccountMeta> = vec![];
    for _ in 0..num_accounts {
        accounts.push(IAccountMeta {
            pubkey: Pubkey::default(),
            signer: false,
            writable: false,
        });
    }
    accounts
}

#[program]
pub mod benchmark_aar {
    use additional_accounts_request::{call, call_faster, call_preflight_interface_function};
    use anchor_lang::solana_program::{log::sol_log_compute_units, program::get_return_data};

    use super::*;

    pub fn preflight_raw_example(ctx: Context<ExampleOne>, num_accounts: u32) -> Result<()> {
        let accounts = create_accounts(num_accounts);
        set_return_data(
            &PreflightPayload {
                accounts,
                has_more: false,
            }
            .try_to_vec()?,
        );
        Ok(())
    }

    pub fn raw_example(ctx: Context<ExampleOne>, num_accounts: u32) -> Result<()> {
        let accounts = create_accounts(num_accounts);
        Ok(())
    }

    pub fn preflight_transfer<'info>(
        ctx: Context<'_, '_, '_, 'info, Callee<'info>>,
        num_accounts: u32,
    ) -> Result<()> {
        call_preflight_interface_function(
            "transfer".to_string(),
            &CpiContext::new(ctx.accounts.program.clone(), Empty {})
                .with_remaining_accounts(ctx.remaining_accounts.to_vec()),
            &num_accounts.try_to_vec()?,
        )?;
        let data = get_return_data().unwrap();
        assert!(data.0 == *ctx.accounts.program.key, "Wrong program id");
        set_return_data(&data.1);
        Ok(())
    }

    pub fn transfer<'info>(
        ctx: Context<'_, '_, '_, 'info, Callee<'info>>,
        num_accounts: u32,
    ) -> Result<()> {
        msg!("Executing transfer: {} accounts...", num_accounts);
        sol_log_compute_units();

        let info = CpiContext::new(ctx.accounts.program.clone(), Empty {})
            .with_remaining_accounts(ctx.remaining_accounts.to_vec());
        msg!("Signer seeds: {:?}", info.signer_seeds);
        call_faster(
            "transfer".to_string(),
            ctx.accounts.program.key(),
            vec![],
            vec![],
            ctx.remaining_accounts,
            info.signer_seeds,
            num_accounts.try_to_vec()?,
            false,
        )?;
        msg!("Finished transfer...");
        sol_log_compute_units();
        Ok(())
    }
}

#[derive(Accounts)]
pub struct ExampleOne {}

#[derive(Accounts)]
pub struct Empty {}

#[derive(Accounts)]
pub struct Callee<'info> {
    /// CHECK: checked by CPI
    program: AccountInfo<'info>,
}

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct ExternalIAccountMeta {
    pubkey: Pubkey,
    signer: bool,
    writable: bool,
}
