use additional_accounts_request::{
    call, identify_additional_accounts, resolve_additional_accounts, InterfaceInstruction,
};
use anchor_lang::{prelude::*, solana_program::program::set_return_data, Discriminator};
use callee::{
    interface::instructions::{ITransfer, ITransferLinkedList, ITransferOwnershipList},
    state::{Node, OwnershipList},
};

#[derive(Accounts)]
pub struct Transfer<'info> {
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
    ctx: Context<'_, '_, '_, 'info, Transfer<'info>>,
    page: u8,
) -> Result<()> {
    let mut args = ctx.accounts.destination.key.try_to_vec().unwrap();
    args.extend(page.to_le_bytes().to_vec());

    let ix_name: String;
    {
        let account_disc = &ctx.accounts.object.try_borrow_data()?[0..8];
        if account_disc == Node::discriminator() {
            ix_name = ITransferLinkedList::instruction_name();
            msg!("Linked list");
        } else if account_disc == OwnershipList::discriminator() {
            ix_name = ITransferOwnershipList::instruction_name();
        } else {
            msg!("Unknown account discriminator");
            return Err(ProgramError::InvalidAccountData.into());
        }
    }

    // The reason to do this is to properly forward other pages of accounts
    // (if at any point more than 29 accounts are used, which is 100% more of a challenge than I expect to be useful)
    let additional_accounts = resolve_additional_accounts(
        ix_name,
        &CpiContext::new(
            ctx.accounts.program.clone(),
            ITransfer {
                owner: ctx.accounts.owner.clone(),
                object: ctx.accounts.object.clone(),
            },
        )
        .with_remaining_accounts(ctx.remaining_accounts.to_vec()),
        &args,
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

pub fn transfer<'info>(ctx: Context<'_, '_, '_, 'info, Transfer<'info>>) -> Result<()> {
    let cpi_ctx = CpiContext::new(
        ctx.accounts.program.clone(),
        ITransfer {
            owner: ctx.accounts.owner.clone(),
            object: ctx.accounts.object.clone(),
        },
    )
    .with_remaining_accounts(ctx.remaining_accounts.to_vec());

    let ix_name: String;
    {
        let account_disc = &ctx.accounts.object.try_borrow_data()?[0..8];
        if account_disc == Node::discriminator() {
            ix_name = ITransferLinkedList::instruction_name();
            msg!("linked list");
        } else if account_disc == OwnershipList::discriminator() {
            ix_name = ITransferOwnershipList::instruction_name();
            msg!("ownership list");
        } else {
            msg!("Unknown account discriminator");
            return Err(ProgramError::InvalidAccountData.into());
        }
    }

    call(
        ix_name,
        cpi_ctx,
        ctx.accounts.destination.key.try_to_vec().unwrap(),
        true,
    )?;
    Ok(())
}
