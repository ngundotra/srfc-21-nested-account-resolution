use additional_accounts_request::{
    call, forward_return_data, identify_additional_accounts, AdditionalAccountsRequest,
    InterfaceInstruction,
};
use anchor_lang::{prelude::*, solana_program::program::set_return_data};
use caller::interface::instructions::ITransferAnything;

#[derive(Accounts)]
pub struct Transfer<'info> {
    /// CHECK: this is the program that actually makes the transfer call
    delegate_program: AccountInfo<'info>,
    /// CHECK:
    program: AccountInfo<'info>,
    /// CHECK:
    owner: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    head: AccountInfo<'info>,
    /// CHECK:
    destination: AccountInfo<'info>,
}

pub fn preflight_transfer<'info>(
    ctx: Context<'_, '_, 'info, 'info, Transfer<'info>>,
    page: u8,
) -> Result<()> {
    let mut additional_accounts = identify_additional_accounts(
        ITransferAnything::instruction_name(),
        &CpiContext::new(
            ctx.accounts.delegate_program.clone(),
            ITransferAnything {
                program: ctx.accounts.program.clone(),
                owner: ctx.accounts.owner.clone(),
                object: ctx.accounts.head.clone(),
                destination: ctx.accounts.destination.clone(),
            },
        )
        .with_remaining_accounts(ctx.remaining_accounts.to_vec()),
        &[],
        false,
    )?;

    additional_accounts.page_to(page)?;
    set_return_data(&additional_accounts.try_to_vec().unwrap());

    Ok(())
}

pub fn transfer<'info>(ctx: Context<'_, '_, 'info, 'info, Transfer<'info>>) -> Result<()> {
    let cpi_ctx = CpiContext::new(
        ctx.accounts.delegate_program.clone(),
        ITransferAnything {
            program: ctx.accounts.program.to_account_info(),
            owner: ctx.accounts.owner.clone(),
            object: ctx.accounts.head.clone(),
            destination: ctx.accounts.destination.clone(),
        },
    )
    .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    call(
        ITransferAnything::instruction_name(),
        cpi_ctx,
        vec![],
        false,
    )?;
    Ok(())
}
