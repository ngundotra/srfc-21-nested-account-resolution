use additional_accounts_request::{
    call, forward_return_data, identify_additional_accounts, resolve_additional_accounts,
    AdditionalAccountsRequest, InterfaceInstruction,
};
use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::set_return_data},
};
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
    object: AccountInfo<'info>,
    /// CHECK:
    destination: AccountInfo<'info>,
}

pub fn preflight_transfer<'info>(
    ctx: Context<'_, '_, 'info, 'info, Transfer<'info>>,
    page: u8,
) -> Result<()> {
    let additional_accounts = resolve_additional_accounts(
        ITransferAnything::instruction_name(),
        &CpiContext::new(
            ctx.accounts.delegate_program.clone(),
            ITransferAnything {
                program: ctx.accounts.program.clone(),
                owner: ctx.accounts.owner.clone(),
                object: ctx.accounts.object.clone(),
                destination: ctx.accounts.destination.clone(),
            },
        )
        .with_remaining_accounts(ctx.remaining_accounts.to_vec()),
        &[],
        page,
        false,
    )?;

    if page as u32 > additional_accounts.num_accounts {
        msg!("Page {} is out of bounds", page);
        return Err(ProgramError::InvalidInstructionData.into());
    }

    set_return_data(bytemuck::bytes_of(&additional_accounts));

    Ok(())
}

pub fn transfer<'info>(ctx: Context<'_, '_, 'info, 'info, Transfer<'info>>) -> Result<()> {
    let cpi_ctx = CpiContext::new(
        ctx.accounts.delegate_program.clone(),
        ITransferAnything {
            program: ctx.accounts.program.to_account_info(),
            owner: ctx.accounts.owner.clone(),
            object: ctx.accounts.object.clone(),
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
