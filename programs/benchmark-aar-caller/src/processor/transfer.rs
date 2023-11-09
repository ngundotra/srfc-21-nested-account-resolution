use additional_accounts_request::{call, forward_return_data, identify_additional_accounts};
use anchor_lang::prelude::*;
use benchmark_aar_callee::interface::instructions::ITransfer;

#[derive(Accounts)]
pub struct Transfer<'info> {
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
    ctx: Context<'_, '_, '_, 'info, Transfer<'info>>,
    page: u8,
) -> Result<()> {
    msg!("Preflighting...");
    ctx.remaining_accounts.iter().for_each(|account| {
        msg!("> account: {}", account.key);
    });

    let mut args = ctx.accounts.destination.key.try_to_vec().unwrap();
    args.extend(page.to_le_bytes().to_vec());

    identify_additional_accounts(
        "transfer_linked_list".to_string(),
        &CpiContext::new(
            ctx.accounts.program.clone(),
            ITransfer {
                owner: ctx.accounts.owner.to_account_info(),
                head_node: ctx.accounts.head.clone(),
            },
        )
        .with_remaining_accounts(ctx.remaining_accounts.to_vec()),
        &args,
        false,
    )?;
    msg!("...preflighted");

    forward_return_data(ctx.accounts.program.key);
    Ok(())
}

pub fn transfer<'info>(ctx: Context<'_, '_, '_, 'info, Transfer<'info>>) -> Result<()> {
    let cpi_ctx = CpiContext::new(
        ctx.accounts.program.clone(),
        ITransfer {
            owner: ctx.accounts.owner.to_account_info(),
            head_node: ctx.accounts.head.clone(),
        },
    )
    .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    call(
        "transfer_linked_list".to_string(),
        cpi_ctx,
        ctx.accounts.destination.key.try_to_vec().unwrap(),
        false,
    )?;
    Ok(())
}
